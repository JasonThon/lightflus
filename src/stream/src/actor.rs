use crate::err::SinkException;
use common::collections::lang;
use common::event::LocalEvent;
use common::net::PersistableHostAddr;
use common::types::{ExecutorId, SinkId, SourceId};
use common::{err, utils};
use proto::common::common::JobId;
use proto::common::stream::{DataflowMeta, OperatorInfo};
use proto::worker::worker::DispatchDataEventStatusEnum;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::thread::JoinHandle;

pub type EventReceiver<Input> = crossbeam_channel::Receiver<Input>;
pub type EventSender<Input> = crossbeam_channel::Sender<Input>;

#[derive(Clone, Debug, PartialEq)]
pub struct DataflowContext {
    pub job_id: JobId,
    pub meta: Vec<DataflowMeta>,
    pub nodes: BTreeMap<ExecutorId, OperatorInfo>,
}

impl DataflowContext {
    pub fn dispatch(&self) -> Result<(), err::CommonException> {
        Ok(())
    }

    pub fn new(
        job_id: JobId,
        meta: Vec<DataflowMeta>,
        nodes: BTreeMap<ExecutorId, OperatorInfo>,
    ) -> DataflowContext {
        DataflowContext {
            job_id,
            meta,
            nodes,
        }
    }

    pub fn create_executors(&self) -> Vec<ExecutorImpl> {
        let ref mut source_sink_manager = SourceSinkManger::new();
        self.meta.iter().for_each(|meta| {
            source_sink_manager.create_local(&(meta.center as ExecutorId));

            meta.neighbors.iter().for_each(|node_id| {
                let executor_id = *node_id as ExecutorId;
                let remote_node = self
                    .nodes
                    .get(&executor_id)
                    .filter(|operator| utils::is_remote_operator(*operator));

                if remote_node.is_some() {
                    remote_node.iter().for_each(|operator| {
                        source_sink_manager.create_remote_sink(&executor_id, *operator)
                    })
                } else {
                    source_sink_manager.create_local(&executor_id)
                }
            })
        });

        self.meta
            .iter()
            .map(|meta| {
                ExecutorImpl::Local(LocalExecutor::with_source_and_sink(
                    meta.center as ExecutorId,
                    source_sink_manager.get_sinks_by_ids(meta.neighbors.clone()),
                    source_sink_manager
                        .get_source_by_id(meta.center.clone())
                        .unwrap(),
                    self.nodes.get(&meta.center).unwrap().clone(),
                ))
            })
            .collect()
    }

    pub fn validate(&self) -> bool {
        return true;
    }
}

pub trait Executor {
    fn run(self) -> JoinHandle<()>;
    fn as_sinkable(&self) -> SinkImpl;
}

#[derive(Clone)]
pub struct LocalExecutor {
    pub executor_id: ExecutorId,
    pub(crate) operator: OperatorInfo,

    sinks: Vec<SinkImpl>,
    local_source: SourceImpl,
}

impl LocalExecutor {
    pub fn with_source_and_sink(
        executor_id: ExecutorId,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
        operator: OperatorInfo,
    ) -> Self {
        Self {
            executor_id,
            operator,
            local_source: source,
            sinks,
        }
    }
}

impl Executor for LocalExecutor {
    fn run(self) -> JoinHandle<()> {
        std::thread::spawn(move || 'outloop: loop {
            while let Some(msg) = self.local_source.fetch_msg() {
                match &msg {
                    SinkableMessageImpl::LocalMessage(message) => match message {
                        LocalEvent::RowChangeStream(change) => {
                            println!("{:?}", change)
                        }
                        LocalEvent::Terminate { job_id, to: _ } => {
                            println!("stopping......");
                            self.sinks.iter().for_each(|sink| {
                                let _ = sink.sink(SinkableMessageImpl::LocalMessage(
                                    LocalEvent::Terminate {
                                        job_id: job_id.clone(),
                                        to: sink.sink_id(),
                                    },
                                ));
                            });
                            break 'outloop;
                        }
                    },
                }
            }
        })
    }

    fn as_sinkable(&self) -> SinkImpl {
        SinkImpl::Local(LocalSink {
            sender: self.local_source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }
}

#[derive(Clone)]
pub enum ExecutorImpl {
    Local(LocalExecutor),
}

impl ExecutorImpl {
    pub fn run(self) -> JoinHandle<()> {
        match self {
            ExecutorImpl::Local(exec) => exec.run(),
        }
    }

    pub fn as_sinkable(&self) -> SinkImpl {
        match self {
            ExecutorImpl::Local(exec) => exec.as_sinkable(),
        }
    }
}

pub trait Source {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl>;
    fn fetch_msg(&self) -> Option<SinkableMessageImpl>;
    fn source_id(&self) -> SourceId;
}

pub struct SourceSinkManger {
    local_source_rx: HashMap<SourceId, Arc<EventReceiver<SinkableMessageImpl>>>,
    sources: HashMap<SourceId, SourceImpl>,
    local_source_tx: HashMap<SourceId, EventSender<SinkableMessageImpl>>,
    sinks: HashMap<SinkId, SinkImpl>,
}

impl SourceSinkManger {
    pub(crate) fn get_source_by_id(&self, source_id: SourceId) -> Option<SourceImpl> {
        self.sources.get(&source_id).map(|s| s.clone())
    }

    pub(crate) fn get_sinks_by_ids(&self, sink_ids: Vec<SinkId>) -> Vec<SinkImpl> {
        self.sinks
            .iter()
            .filter(|entry| lang::any_match(&sink_ids, |id| id == entry.0))
            .map(|entry| entry.1.clone())
            .collect()
    }

    /*
    call this method will create local sink and inner source simultaneously
     */
    pub(crate) fn create_local(&mut self, executor_id: &ExecutorId) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let rx_arc = Arc::new(rx);
        self.local_source_rx
            .insert(*executor_id as SourceId, rx_arc.clone());
        self.local_source_tx
            .insert(*executor_id as SourceId, tx.clone());
        let source = LocalSource {
            recv: rx_arc,
            source_id: executor_id.clone(),
            tx: tx.clone(),
        };

        self.sources.insert(*executor_id, SourceImpl::Local(source));
        self.sinks.insert(
            *executor_id,
            SinkImpl::Local(LocalSink {
                sender: tx,
                sink_id: *executor_id,
            }),
        );
    }

    pub(crate) fn create_remote_sink(&mut self, executor_id: &ExecutorId, info: &OperatorInfo) {
        self.local_source_tx.get(executor_id).iter().for_each(|tx| {
            self.sinks.insert(
                *executor_id,
                SinkImpl::Remote(RemoteSink {
                    sink_id: *executor_id,
                    host_addr: PersistableHostAddr {
                        host: "".to_string(),
                        port: 0,
                    },
                }),
            );
        })
    }
}

impl SourceSinkManger {
    fn new() -> SourceSinkManger {
        SourceSinkManger {
            local_source_rx: Default::default(),
            sources: Default::default(),
            local_source_tx: Default::default(),
            sinks: Default::default(),
        }
    }
}

pub trait Sink {
    fn sink_id(&self) -> SinkId;
    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException>;
}

pub trait SinkableMessage {}

#[derive(Clone, Debug)]
pub enum SinkableMessageImpl {
    LocalMessage(LocalEvent),
}

impl SinkableMessage for SinkableMessageImpl {}

unsafe impl Send for SinkableMessageImpl {}
unsafe impl Sync for SinkableMessageImpl {}

#[derive(Clone)]
pub struct LocalSink {
    pub(crate) sender: EventSender<SinkableMessageImpl>,
    pub(crate) sink_id: SinkId,
}

impl Sink for LocalSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match &msg {
            SinkableMessageImpl::LocalMessage(_) => self
                .sender
                .send(msg)
                .map(|_| DispatchDataEventStatusEnum::DONE)
                .map_err(|err| err.into()),
            _ => Err(SinkException::invalid_message_type()),
        }
    }
}

#[derive(Clone)]
pub struct RemoteSink {
    pub(crate) sink_id: SinkId,
    pub(crate) host_addr: PersistableHostAddr,
}

impl Sink for RemoteSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    /**
    Generally, remote/any database sink has to support three kinds of message delivery:
    1. At most once
    2. At least once
    3. Exactly once
    Each one face different tech trade-off. In 1.0.* version, we only support At Least Once sink in default.
    From 2.0 version, 2-PC exactly-once delivery will be planed to be supported.
    **/
    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        todo!()
    }
}

#[derive(Clone)]
pub struct LocalSource {
    pub(crate) recv: Arc<EventReceiver<SinkableMessageImpl>>,
    pub(crate) source_id: SourceId,
    tx: EventSender<SinkableMessageImpl>,
}

impl Source for LocalSource {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        self.tx.clone()
    }

    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        self.recv.recv().ok()
    }

    fn source_id(&self) -> SourceId {
        self.source_id
    }
}

#[derive(Clone)]
pub enum SourceImpl {
    Local(LocalSource),
}

impl SourceImpl {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.create_msg_sender(),
        }
    }

    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.fetch_msg(),
        }
    }

    fn source_id(&self) -> SourceId {
        match self {
            SourceImpl::Local(source) => source.source_id(),
        }
    }
}

#[derive(Clone)]
pub enum SinkImpl {
    Local(LocalSink),
    Remote(RemoteSink),
}

impl Sink for SinkImpl {
    fn sink_id(&self) -> SinkId {
        match self {
            SinkImpl::Local(sink) => sink.sink_id(),
            SinkImpl::Remote(sink) => sink.sink_id(),
        }
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg),
            SinkImpl::Remote(sink) => sink.sink(msg),
        }
    }
}

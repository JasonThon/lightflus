use crate::err::SinkException;
use common::event::LocalEvent;
use common::net::HostAddr;
use common::types::{ExecutorId, SinkId, SourceId};
use common::{err, utils};
use proto::common::common::JobId;
use proto::common::stream::{DataflowMeta, OperatorInfo};
use proto::worker::worker::DispatchDataEventStatusEnum;
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub type EventReceiver<Input> = mpsc::UnboundedReceiver<Input>;
pub type EventSender<Input> = mpsc::UnboundedSender<Input>;

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
                        self.nodes.get(&executor_id).iter().for_each(|info| {
                            source_sink_manager.create_remote_sink(&executor_id, *info)
                        })
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
                    source_sink_manager
                        .get_sinks_by_ids(meta.neighbors.clone())
                        .iter()
                        .map(|sink| sink.clone())
                        .collect(),
                    source_sink_manager
                        .get_source_by_id(meta.center.clone())
                        .unwrap(),
                ))
            })
            .collect()
    }

    pub fn validate(&self) -> bool {
        return true;
    }
}

pub trait Executor {
    fn run(&self) -> JoinHandle<()>;
    fn as_sinkable(&self) -> SinkImpl;
}

pub struct LocalExecutor {
    pub executor_id: ExecutorId,

    sinks: Vec<SinkImpl>,
    local_source: Arc<SourceImpl>,
}

impl LocalExecutor {
    pub fn with_source_and_sink(
        executor_id: ExecutorId,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
    ) -> Self {
        Self {
            executor_id,
            local_source: Arc::new(source),
            sinks,
        }
    }
}

impl Executor for LocalExecutor {
    fn run(&self) -> JoinHandle<()> {
        todo!()
    }

    fn as_sinkable(&self) -> SinkImpl {
        SinkImpl::Local(LocalSink {
            sender: self.local_source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }
}

pub enum ExecutorImpl {
    Local(LocalExecutor),
}

impl Executor for ExecutorImpl {
    fn run(&self) -> JoinHandle<()> {
        match self {
            ExecutorImpl::Local(exec) => exec.run(),
        }
    }

    fn as_sinkable(&self) -> SinkImpl {
        match self {
            ExecutorImpl::Local(exec) => exec.as_sinkable(),
        }
    }
}

pub trait Source {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl>;
}

pub struct SourceSinkManger {
    local_sink_id_set: HashSet<SinkId>,
    remote_sinks_infos: HashMap<SinkId, OperatorInfo>,
    local_source_rx: RefCell<HashMap<SourceId, Arc<EventReceiver<SinkableMessageImpl>>>>,
    sources: RefCell<HashMap<SourceId, SourceImpl>>,
    local_source_tx: RefCell<HashMap<SourceId, EventSender<SinkableMessageImpl>>>,
    sinks: RefCell<HashMap<SinkId, SinkImpl>>,
}

impl SourceSinkManger {
    pub(crate) fn get_source_by_id(&self, source_id: SourceId) -> Option<SourceImpl> {
        let borrowed_local_sources = self.sources.borrow();
        if borrowed_local_sources.contains_key(&source_id) {
            return borrowed_local_sources.get(&source_id).map(|s| s.clone());
        }

        self.local_source_rx.borrow().get(&source_id).map(|recv| {
            let source = LocalSource {
                recv: recv.clone(),
                source_id: source_id.clone(),
                tx: self
                    .local_source_tx
                    .borrow()
                    .get(&source_id)
                    .map(|tx| tx.clone())
                    .unwrap(),
            };

            RefMut::map(self.sources.borrow_mut(), |map| {
                map.insert(source_id, SourceImpl::Local(source.clone()));
                map
            });

            SourceImpl::Local(source)
        })
    }

    pub(crate) fn get_sinks_by_ids(&self, sink_ids: Vec<SinkId>) -> Vec<SinkImpl> {
        sink_ids
            .iter()
            .map(|sink_id| {
                let borrowed_local_sinks = self.sinks.borrow();
                if borrowed_local_sinks.contains_key(sink_id) {
                    return borrowed_local_sinks
                        .get(sink_id)
                        .map(|b| b.clone())
                        .unwrap();
                }

                if self.local_sink_id_set.contains(sink_id) {
                    let (tx, rx) = mpsc::unbounded_channel();
                    let _ = RefMut::map(self.local_source_rx.borrow_mut(), |map| {
                        map.insert(*sink_id as SourceId, Arc::new(rx));
                        map
                    });
                    let _ = RefMut::map(self.local_source_tx.borrow_mut(), |map| {
                        map.insert(*sink_id as SourceId, tx.clone());
                        map
                    });

                    SinkImpl::Local(LocalSink {
                        sender: tx,
                        sink_id: *sink_id,
                    })
                } else {
                    SinkImpl::Remote(RemoteSink {
                        sink_id: *sink_id,
                        host_addr: HostAddr {
                            host: "".to_string(),
                            port: 0,
                        },
                    })
                }
            })
            .collect()
    }

    /*
    create local sink and inner source
     */
    pub(crate) fn create_local(&mut self, executor_id: &ExecutorId) {
        self.local_sink_id_set.insert(*executor_id as SinkId);
    }

    pub(crate) fn create_remote_sink(&mut self, executor_id: &ExecutorId, info: &OperatorInfo) {
        self.remote_sinks_infos
            .insert(*executor_id as SinkId, info.clone());
    }
}

impl SourceSinkManger {
    fn new() -> SourceSinkManger {
        SourceSinkManger {
            local_sink_id_set: Default::default(),
            remote_sinks_infos: Default::default(),
            local_source_rx: RefCell::new(Default::default()),
            sources: RefCell::new(Default::default()),
            local_source_tx: RefCell::new(Default::default()),
            sinks: RefCell::new(Default::default()),
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
    pub(crate) host_addr: HostAddr,
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
}

#[derive(Clone)]
pub enum SourceImpl {
    Local(LocalSource),
}

impl Source for SourceImpl {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.create_msg_sender(),
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

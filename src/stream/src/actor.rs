use common::types::{ExecutorId, SinkId, SourceId};
use std::sync;
use common::{err, types};
use proto::common::common::JobId;
use proto::common::stream::{DataflowMeta, OperatorInfo};
use std::collections::{BTreeMap, HashMap, HashSet};
use proto::common::stream;
use std::cell::{RefCell, RefMut};
use std::sync::Arc;
use std::any::{Any, TypeId};
use tokio::sync::mpsc;
use common::event::LocalEvent;
use tokio::task::JoinHandle;
use common::net::HostAddr;
use crate::err::SinkException;
use crate::{trigger, window};

pub type EventReceiver<Input> = mpsc::UnboundedReceiver<Input>;
pub type EventSender<Input> = mpsc::UnboundedSender<Input>;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StreamConfig {
    // trigger type
    pub trigger_type: trigger::TriggerType,
    // window
    pub window_type: window::WindowType,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DataflowContext {
    pub job_id: JobId,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub meta: Vec<DataflowMeta>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    #[serde(default)]
    pub nodes: BTreeMap<ExecutorId, stream::OperatorInfo>,
    pub config: StreamConfig,
}

impl DataflowContext {
    pub fn dispatch(&self) -> Result<(), err::CommonException> {
        Ok(())
    }

    pub fn new(job_id: JobId,
               meta: Vec<DataflowMeta>,
               nodes: BTreeMap<ExecutorId, stream::OperatorInfo>,
               config: StreamConfig) -> DataflowContext {
        DataflowContext {
            job_id,
            meta,
            nodes,
            config,
        }
    }

    pub fn create_executors(&self) -> Vec<dyn Executor> {
        let ref mut source_sink_manager = SourceSinkManger::new();
        self.meta
            .iter()
            .for_each(|meta| {
                source_sink_manager.create_local(&(meta.center as ExecutorId));

                meta.neighbors
                    .iter()
                    .for_each(|node_id| {
                        let executor_id = node_id as ExecutorId;

                        if self.nodes.contains_key(&executor_id) {
                            source_sink_manager.create_local(&executor_id)
                        } else {
                            self.nodes.get(&executor_id)
                                .iter()
                                .for_each(|info| source_sink_manager.create_remote_sink(&executor_id, *info))
                        }
                    })
            });

        self.meta
            .iter()
            .map(|meta| LocalExecutor::with_source_and_sink(
                meta.center as ExecutorId,
                source_sink_manager
                    .get_sinks_by_ids(meta.neighbors.clone())
                    .iter()
                    .map(|sink| sync::Mutex::new(**sink))
                    .collect(),
                source_sink_manager
                    .get_source_by_id(meta.center.clone())
                    .unwrap(),
            ))
            .collect()
    }
}

pub trait Executor {
    fn run(&self) -> JoinHandle<()>;
    fn as_sinkable(&self) -> Box<dyn Sink>;
}

pub struct LocalExecutor {
    pub executor_id: ExecutorId,

    sinks: Vec<sync::Mutex<dyn Sink>>,
    source: sync::Arc<Box<dyn Source>>,
}

impl LocalExecutor {
    pub fn with_source_and_sink(executor_id: ExecutorId,
                                sinks: Vec<sync::Mutex<dyn Sink>>,
                                source: Box<dyn Source>) -> Self {
        Self {
            executor_id,
            source: sync::Arc::new(source),
            sinks,
        }
    }
}

impl Executor for LocalExecutor {
    fn run(&self) -> JoinHandle<()> {
        todo!()
    }

    fn as_sinkable(&self) -> Box<dyn Sink> {
        Box::new(LocalSink {
            sender: self.source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }
}

pub trait Source {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl>;
}

#[derive(Clone)]
pub struct SourceSinkManger {
    local_sink_id_set: HashSet<SinkId>,
    remote_sinks_infos: HashMap<SinkId, OperatorInfo>,
    local_source_rx: RefCell<HashMap<SourceId, Arc<EventReceiver<SinkableMessageImpl>>>>,
    local_sources: RefCell<HashMap<SourceId, Box<dyn Source>>>,
    local_source_tx: RefCell<HashMap<SourceId, EventSender<SinkableMessageImpl>>>,
    local_sinks: RefCell<HashMap<SinkId, Box<dyn Sink>>>,
}

impl SourceSinkManger {
    pub(crate) fn get_source_by_id(&self, source_id: types::SourceId) -> Option<Box<dyn Source>> {
        let borrowed_local_sources = self.local_sources.borrow();
        if borrowed_local_sources.contains_key(&source_id) {
            return borrowed_local_sources.get(&source_id)
                .map(|s| *s);
        }

        self.local_source_rx.borrow()
            .get(&source_id)
            .map(|recv| {
                let boxed_source = Box::new(LocalSource {
                    recv: recv.clone(),
                    source_id: source_id.clone(),
                    tx: self.local_source_tx
                        .borrow()
                        .get(&source_id)
                        .map(|tx| tx.clone())
                        .unwrap(),
                });

                RefMut::map(
                    self.local_sources.borrow_mut(),
                    |map| {
                        map.insert(source_id, boxed_source.clone());
                        map
                    },
                );

                boxed_source
            })
    }

    pub(crate) fn get_sinks_by_ids(&self, sink_ids: Vec<types::SinkId>) -> Vec<Box<dyn Sink>> {
        sink_ids
            .iter()
            .map(|sink_id| {
                let borrowed_local_sinks = self.local_sinks.borrow();
                if borrowed_local_sinks.contains_key(sink_id) {
                    return borrowed_local_sinks
                        .get(sink_id)
                        .map(|b| *b)
                        .unwrap();
                }

                if self.local_sink_id_set.contains(sink_id) {
                    let (tx, rx) = mpsc::unbounded_channel();
                    let _ = RefMut::map(
                        self.local_source_rx.borrow_mut(),
                        |map| {
                            map.insert(*sink_id as SourceId, Arc::new(rx));
                            map
                        },
                    );
                    let _ = RefMut::map(
                        self.local_source_tx.borrow_mut(),
                        |map| {
                            map.insert(*sink_id as SourceId, tx.clone());
                            map
                        },
                    );

                    Box::new(SinkImpl::Local(
                        LocalSink {
                            sender: tx,
                            sink_id: *sink_id,
                        })
                    )
                } else {
                    Box::new(SinkImpl::Remote(
                        RemoteSink {
                            sink_id: *sink_id,
                            host_addr: HostAddr { host: "".to_string(), port: 0 },
                        })
                    )
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
        self.remote_sinks_infos.insert(*executor_id as SinkId, info.clone());
    }
}

impl SourceSinkManger {
    fn new() -> SourceSinkManger {
        SourceSinkManger {
            local_sink_id_set: Default::default(),
            remote_sinks_infos: Default::default(),
            local_source_rx: RefCell::new(Default::default()),
            local_sources: RefCell::new(Default::default()),
            local_source_tx: RefCell::new(Default::default()),
            local_sinks: RefCell::new(Default::default()),
        }
    }
}

pub trait Sink {
    fn sink_id(&self) -> types::SinkId;
    fn sink<M: SinkableMessage>(&self, msg: M) -> Result<(), SinkException>;
}

pub trait SinkableMessage {}

#[derive(Clone, Eq, PartialEq)]
pub enum SinkableMessageImpl {
    LocalMessage(LocalEvent),
}

impl SinkableMessage for SinkableMessageImpl {}

#[derive(Clone)]
pub struct LocalSink {
    pub(crate) sender: EventSender<SinkableMessageImpl>,
    pub(crate) sink_id: types::SinkId,
}

impl Sink for LocalSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    fn sink<M: SinkableMessage>(&self, msg: M) -> Result<(), SinkException> {
        if msg.type_id() == &TypeId::of::<SinkableMessageImpl>() {
            self.sender
                .send(msg as SinkableMessageImpl)
                .map_err(|err| err.into())
        } else {
            Err(SinkException::invalid_message_type())
        }
    }
}

#[derive(Clone)]
pub struct RemoteSink {
    pub(crate) sink_id: types::SinkId,
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
    fn sink<M: SinkableMessage>(&self, msg: M) -> Result<(), SinkException> {
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

pub(crate) enum SinkImpl {
    Local(LocalSink),
    Remote(RemoteSink),
}

impl Sink for SinkImpl {
    fn sink_id(&self) -> SinkId {
        match self {
            SinkImpl::Local(sink) => sink.sink_id(),
            SinkImpl::Remote(sink) => sink.sink_id()
        }
    }

    fn sink<M: SinkableMessage>(&self, msg: M) -> Result<(), SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg),
            SinkImpl::Remote(sink) => sink.sink(msg)
        }
    }
}

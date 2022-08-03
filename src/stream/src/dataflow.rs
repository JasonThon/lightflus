use std::{collections, marker, sync};
use std::collections::BTreeMap;
use std::hash::Hash;
use rayon::prelude::*;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use common::{err, event, types};
use common::err::ExecutionException;
use common::event::LocalEvent;
use common::types::{ExecutorId, JobID, NodeIdx, NodeSet};
use proto::common::common::JobId;
use proto::common::stream;
use proto::common::stream::{DataflowMeta, OperatorInfo};
use crate::{pipeline, state::StateManager, trigger, window};

pub fn new_event_pipe<T>() -> (EventSender<T>, EventReceiver<T>) {
    mpsc::unbounded_channel::<T>()
}

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
        let ref mut sink_manager = SinkManger::new();
        let ref mut source_manager = SourceManager::new();
        self.meta
            .iter()
            .for_each(|meta| {
                sink_manager.create_local(&(meta.center as ExecutorId));

                meta.neighbors
                    .iter()
                    .for_each(|node_id| {
                        let executor_id = node_id as ExecutorId;
                        if self.nodes.contains_key(&executor_id) {
                            sink_manager.create_local(&executor_id)
                        } else {
                            self.nodes.get(&executor_id)
                                .iter()
                                .for_each(|info| sink_manager.create_remote(&executor_id, *info))
                        }
                    })
            }
            );

        self.meta
            .iter()
            .map(|meta| LocalExecutor::with_source_and_sink(
                meta.center as ExecutorId,
                sink_manager
                    .get_by_ids(meta.neighbors.clone())
                    .iter()
                    .map(|sink| sync::Mutex::new(**sink))
                    .collect(),
                source_manager.get_by_id(meta.center.clone()),
            ))
            .collect()
    }
}

type SourceManagerRef = sync::Arc<SourceManager>;

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
        todo!()
    }
}

pub struct SourceManager {}

impl SourceManager {
    pub fn new() -> SourceManager {
        SourceManager {}
    }
    pub(crate) fn get_by_id(&self, id: types::SourceId) -> Box<dyn Source> {
        todo!()
    }
}

pub trait Source {}

#[derive(Clone)]
pub struct SinkManger {}

impl SinkManger {
    pub(crate) fn get_by_ids(&self, sink_ids: Vec<types::SinkId>) -> Vec<Box<dyn Sink>> {
        todo!()
    }

    pub(crate) fn create_local(&mut self, executor_id: &ExecutorId) {
        todo!()
    }

    pub(crate) fn create_remote(&mut self, executor_id: &ExecutorId, info: &OperatorInfo) {
        todo!()
    }
}

impl SinkManger {
    fn new() -> SinkManger {
        SinkManger {}
    }
}

pub trait Sink {
    fn sink_id(&self) -> types::SinkId;
    fn sink<M: SinkableMessage>(&self, msg: M) -> Result<(), SinkException>;
}

pub trait SinkableMessage {}

pub enum SinkableMessageImpl {
    LocalMessage(LocalEvent),
}

impl SinkableMessage for SinkableMessageImpl {}

pub struct SinkException {}
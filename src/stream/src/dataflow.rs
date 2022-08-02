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
use proto::common::stream::DataflowMeta;
use crate::{pipeline, state::StateManager, trigger, window};

pub fn new_event_pipe<T>() -> (EventSender<T>, EventReceiver<T>) {
    mpsc::unbounded_channel::<T>()
}

pub type EventReceiver<Input> = mpsc::UnboundedReceiver<Input>;
pub type EventSender<Input> = mpsc::UnboundedSender<Input>;

pub struct DataStream<
    Input,
    Output,
    T,
    P,
    InputKey: Clone + Send + Sync + Eq + Hash + Ord,
    InputValue: Clone,
    StateValue>
    where T: Sink<Output>,
          Input: event::KeyedEvent<InputKey, InputValue>,
          P: pipeline::Executor<InputKey, InputValue, Output, StateValue>,
          StateValue: Clone {
    window: Option<window::WindowType>,
    trigger: Option<trigger::TriggerType>,
    input: marker::PhantomData<Input>,
    output: marker::PhantomData<Output>,
    input_key: marker::PhantomData<InputKey>,
    input_value: marker::PhantomData<InputValue>,
    state_value: marker::PhantomData<StateValue>,
    rx: EventReceiver<Vec<Input>>,
    disconnect: mpsc::Receiver<Close>,
    pipeline: P,
    sink: T,
}

impl<Input, Output, T, P,
    InputKey: Clone + Send + Sync + Eq + Hash + Ord,
    InputValue: Clone,
    StateValue>
DataStream<Input,
    Output,
    T,
    P,
    InputKey,
    InputValue,
    StateValue>
    where T: Sink<Output>,
          Input: event::KeyedEvent<InputKey, InputValue>,
          P: pipeline::Executor<InputKey, InputValue, Output, StateValue>,
          StateValue: Clone {
    pub fn new(
        window_type: window::WindowType,
        trigger: trigger::TriggerType,
        rx: EventReceiver<Vec<Input>>,
        disconnect: mpsc::Receiver<Close>,
        pipeline: P,
        sink: T) ->
        DataStream<Input, Output,
            T, P,
            InputKey, InputValue,
            StateValue> {
        DataStream {
            window: Some(window_type),
            trigger: Some(trigger),
            input: Default::default(),
            output: Default::default(),
            input_key: Default::default(),
            input_value: Default::default(),
            state_value: Default::default(),
            rx,
            disconnect,
            pipeline,
            sink,
        }
    }

    pub async fn start(mut self) {
        let ref mut stream = self;
        let ref mut trigger = match &stream.trigger {
            Some(trigger_type) => trigger::Trigger::from(trigger_type),
            None => trigger::Trigger::default()
        };

        let ref mut windows: collections::VecDeque<window::KeyedWindow<InputKey, InputValue>> = Default::default();
        let assigner: window::KeyedWindowAssigner<InputKey, InputValue, Input> = match &stream.window {
            Some(window_type) => window::window_assigner(window_type),
            None => window::default_assigner()
        };

        let ref context = stream.pipeline.create_context();

        loop {
            tokio::select! {
                Some(closed) = stream.disconnect.recv() => break,
                Some(inputs) = stream.rx.recv() => {
                    let new_windows = common::lists::map_reduce(&inputs, |input| assigner.assign(input));
                    windows.extend(new_windows);
                    assigner.merge(windows);
                }
                true = trigger.trigger() => {
                    if windows.is_empty() {
                        continue
                    }

                    let ref mut group = common::lists::group_deque_btree_map(windows, |win| win.key.clone());
                    group.into_par_iter()
                        .for_each(|(key, windows)| {
                        for win in windows {
                            match stream.pipeline.apply(win, context) {
                                Ok(output) => stream.sink.sink(output),
                                Err(err) => {}
                            }
                        }
                    });
                    trigger.refresh()
                }
                else => continue
            }
        }

        stream.disconnect.close();
        stream.rx.close();
    }
}

pub struct Close;

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

    pub fn create_executors(&self) -> Vec<Box<dyn Executor>> {
        let ref mut sink_manager = SinkManger::new();
        let ref mut source_manager = SourceManager::new();
        self.meta
            .iter()
            .map(|meta| {
                meta.neighbors
                    .iter()
                    .for_each(|node_id| {
                        let executor_id = node_id as ExecutorId;
                        if self.nodes.contains_key(&executor_id) {
                            sink_manager.create_local(&executor_id)
                        } else {
                            sink_manager.create_remote(&executor_id)
                        }
                    });

                LocalExecutor::with_source_and_sink(
                    meta.center as ExecutorId,
                    sink_manager.get_manager_by_ids(meta.neighbors.clone()),
                    source_manager.get_manager_by_id(meta.center.clone()),
                )
            }).for_each()
        todo!()
    }
}

type SourceManagerRef = sync::Arc<SourceManager>;

pub trait Executor {
    fn run(&self) -> JoinHandle<()>;
    fn as_sinkable(&self) -> Box<dyn Sink>;
}

pub struct LocalExecutor {
    pub executor_id: ExecutorId,

    source: SourceManagerRef,
    sink: SinkManger,
    sinks: Vec<Box<dyn Sink>>,
}

impl LocalExecutor {
    pub fn with_source_and_sink(executor_id: ExecutorId,
                                sink: SinkManger,
                                source: SourceManager) -> Self {
        Self {
            executor_id,
            source: sync::Arc::new(source),
            sink,
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
    pub(crate) fn get_manager_by_id(&self, id: types::SourceId) -> SourceManager {
        todo!()
    }
}

pub trait Source {}

#[derive(Clone)]
pub struct SinkManger {}

impl SinkManger {
    pub(crate) fn get_manager_by_ids(&self, sink_ids: Vec<types::SinkId>) -> SinkManger {
        todo!()
    }

    pub(crate) fn create_local(&mut self, executor_id: &ExecutorId) {
        todo!()
    }

    pub(crate) fn create_remote(&mut self, executor_id: &ExecutorId) {
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
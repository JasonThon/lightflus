use std::{collections, marker, sync};
use std::collections::BTreeMap;
use std::hash::Hash;
use rayon::prelude::*;
use tokio::sync::mpsc;
use common::{err, event, types};
use common::err::ExecutionException;
use common::types::{AdjacentList, ExecutorId, JobID, NodeIdx, NodeSet, OperatorInfo};
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
          Input: event::Event<InputKey, InputValue>,
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
          Input: event::Event<InputKey, InputValue>,
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
    pub nodes: BTreeMap<types::ExecutorId, OperatorInfo>,
    pub config: StreamConfig,
}

impl DataflowContext {
    pub fn dispatch(&self) -> Result<(), err::CommonException> {
        let mut group = collections::HashMap::<String, Vec<&OperatorInfo>>::new();

        for (_, operator) in &self.nodes {
            match group.get(&operator.addr) {
                Some(ops) => {
                    let mut new_operators = vec![operator];
                    for x in ops {
                        new_operators.push(*x);
                    }

                    group.insert(operator.addr.clone(), new_operators);
                }
                None => {
                    group.insert(operator.addr.clone(), vec![operator]);
                }
            }
        }

        for (addr, ops) in group {
            let client = dataflow_api::worker::new_dataflow_worker_client(
                dataflow_api::worker::DataflowWorkerConfig {
                    host: None,
                    port: None,
                    uri: Some(addr),
                }
            );

            let ref graph_event = event::GraphEvent::GraphSubmit {
                ctx: (&self.job_id, ops, &self.meta).into(),
            };

            let ref mut request = dataflow_api::dataflow_worker::ActionSubmitRequest::default();
            let result = serde_json::to_vec(graph_event)
                .map_err(|err| err::CommonException::from(err))
                .and_then(|value| {
                    request.set_value(value);
                    client.submit_action(request)
                        .map_err(|err| err::CommonException::from(err))
                })
                .map(|_| {
                    log::debug!("submit success")
                });

            if result.is_err() {
                return result;
            }
        }
        Ok(())
    }

    pub fn new(job_id: JobId,
               meta: Vec<DataflowMeta>,
               nodes: BTreeMap<ExecutorId, OperatorInfo>,
               config: StreamConfig) -> DataflowContext {
        DataflowContext {
            job_id,
            meta,
            nodes,
            config,
        }
    }

    pub fn create_executors(&self) -> Vec<dyn Executor> {
        todo!()
    }
}

type SourceManagerRef = sync::Arc<SourceManager>;

pub trait Executor {}

pub struct LocalExecutor {
    pub executor_id: types::ExecutorId,

    source: sync::Arc<SourceManager>,
    sink: SinkManger,
}

impl LocalExecutor {
    pub fn with_source_and_sink(executor_id: types::ExecutorId,
                                sink: SinkManger,
                                source: SourceManager) -> Self <S, K> {
        Self {
            executor_id,
            source: sync::Arc::new(source),
            sink,
        }
    }
}

impl Executor for LocalExecutor {}

pub struct SourceManager {}

pub trait Source {}

pub struct SinkManger {}

pub trait Sink {}

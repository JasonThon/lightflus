use std::{collections, marker};
use std::hash::Hash;
use rayon::prelude::*;
use tokio::sync::mpsc;
use common::{event, types};
use crate::{state::StateManager, window, trigger, pipeline};

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

pub trait Sink<Output>: Send + Sync {
    fn sink(&self, output: Output);
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct StreamConfig {
    // trigger type
    pub trigger_type: trigger::TriggerType,
    // window
    pub window_type: window::WindowType,
}
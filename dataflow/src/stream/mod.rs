use std::{collections, marker};
use std::hash::Hash;
use rayon::prelude::*;
use tokio::sync::mpsc;
use crate::{event, types};
use state::StateManager;

pub mod window;
pub mod trigger;
pub mod pipeline;
#[cfg(feature = "datastream")]
pub mod state;

pub fn stream_pipe<T>() -> (StreamPipeSender<T>, StreamPipeReceiver<T>) {
    mpsc::unbounded_channel::<T>()
}

pub type StreamPipeReceiver<Input> = mpsc::UnboundedReceiver<Input>;
pub type StreamPipeSender<Input> = mpsc::UnboundedSender<Input>;

pub struct DataStream<
    Input,
    Output,
    T,
    P,
    InputKey: Clone + Send + Sync + Eq + PartialEq + Hash,
    InputValue: Clone,
    StateValue>
    where T: Sink<Output>,
          Input: event::Event<InputKey, InputValue>,
          P: pipeline::Pipeline<InputKey, InputValue, Output, StateValue>,
          StateValue: Clone {
    window: Option<window::WindowType>,
    trigger: Option<trigger::TriggerType>,
    input: marker::PhantomData<Input>,
    output: marker::PhantomData<Output>,
    input_key: marker::PhantomData<InputKey>,
    input_value: marker::PhantomData<InputValue>,
    state_value: marker::PhantomData<StateValue>,
    rx: StreamPipeReceiver<Vec<Input>>,
    disconnect: mpsc::Receiver<Close>,
    pipeline: P,
    sink: T,
}

impl<Input, Output, T, P,
    InputKey: Clone + Send + Sync + Eq + PartialEq + Hash,
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
          P: pipeline::Pipeline<InputKey, InputValue, Output, StateValue>,
          StateValue: Clone {
    pub fn new(
        window_type: window::WindowType,
        trigger: trigger::TriggerType,
        rx: StreamPipeReceiver<Vec<Input>>,
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

        let ref context = pipeline::Context::<InputKey, StateValue>::new();

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

                    let ref mut group = common::lists::group_deque_hashmap(windows, |win| win.key.clone());
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

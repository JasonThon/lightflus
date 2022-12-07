use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
};

use chrono::Duration;
use common::{
    collections::lang,
    types::{NodeIdx, TypedValue},
};
use prost_types::Timestamp;
use proto::common::{
    keyed_data_event, operator_info::Details, Entry, KeyedDataEvent, OperatorInfo,
};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use v8::HandleScope;

use crate::{err::RunnableTaskError, state, v8_runtime::RuntimeEngine, DETAULT_WATERMARK};

pub struct DataflowTask<'s, 'i, S: state::StateManager>
where
    's: 'i,
{
    operator: OperatorImpl<S>,
    rt_engine: RefCell<RuntimeEngine<'s, 'i>>,
}

impl<'s, 'i, S: state::StateManager> DataflowTask<'s, 'i, S>
where
    's: 'i,
{
    pub fn new(
        op_info: &OperatorInfo,
        state_manager: S,
        scope: &'i mut HandleScope<'s, ()>,
    ) -> Self {
        let (rt_engine, operator) = if op_info.clone().details.is_some() {
            let detail = op_info.details.as_ref().unwrap();
            match detail {
                Details::Mapper(map_value) => (
                    RefCell::new(RuntimeEngine::new(
                        &map_value.get_func().function,
                        &get_function_name(op_info),
                        scope,
                    )),
                    OperatorImpl::Map(MapOperator::new(&op_info, state_manager)),
                ),
                Details::Filter(filter_value) => (
                    RefCell::new(RuntimeEngine::new(
                        &filter_value.get_func().function,
                        &get_function_name(op_info),
                        scope,
                    )),
                    OperatorImpl::Filter(FilterOperator::new(op_info, state_manager)),
                ),
                Details::KeyBy(key_by_value) => (
                    RefCell::new(RuntimeEngine::new(
                        &key_by_value.get_func().function,
                        &get_function_name(op_info),
                        scope,
                    )),
                    OperatorImpl::KeyBy(KeyByOperator::new(op_info, state_manager)),
                ),
                Details::Reducer(reduce_value) => (
                    RefCell::new(RuntimeEngine::new(
                        &reduce_value.get_func().function,
                        &get_function_name(op_info),
                        scope,
                    )),
                    OperatorImpl::Reduce(ReduceOperator::new(op_info, state_manager)),
                ),
                Details::FlatMap(flat_map_value) => (
                    RefCell::new(RuntimeEngine::new(
                        &flat_map_value.get_func().function,
                        &get_function_name(op_info),
                        scope,
                    )),
                    OperatorImpl::FlatMap(FlatMapOperator::new(op_info, state_manager)),
                ),
                _ => (
                    RefCell::new(RuntimeEngine::new("", "", scope)),
                    OperatorImpl::Empty,
                ),
            }
        } else {
            (
                RefCell::new(RuntimeEngine::new("", "", scope)),
                OperatorImpl::Empty,
            )
        };
        Self {
            rt_engine,
            operator,
        }
    }

    pub(crate) fn process(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
        self.operator.call_fn(event, &self.rt_engine)
    }
}

fn get_function_name(op_info: &OperatorInfo) -> String {
    match op_info.details.as_ref() {
        Some(info) => match info {
            Details::Mapper(_) => format!("_operator_{}_process", "map"),
            Details::Filter(_) => format!("_operator_{}_process", "filter"),
            Details::KeyBy(_) => format!("_operator_{}_process", "keyBy"),
            Details::Reducer(_) => format!("_operator_{}_process", "reduce"),
            Details::FlatMap(_) => format!("_operator_{}_process", "flatMap"),
            _ => "".to_string(),
        },
        None => "".to_string(),
    }
}

pub(crate) trait IOperator {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i;
}

pub(crate) enum OperatorImpl<S: state::StateManager> {
    Map(MapOperator<S>),
    Filter(FilterOperator<S>),
    KeyBy(KeyByOperator<S>),
    FlatMap(FlatMapOperator<S>),
    Reduce(ReduceOperator<S>),
    Empty,
}

impl<S: state::StateManager> IOperator for OperatorImpl<S> {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i,
    {
        match self {
            Self::Map(op) => op.call_fn(event, rt_engine),
            Self::Filter(op) => op.call_fn(event, rt_engine),
            Self::KeyBy(op) => op.call_fn(event, rt_engine),
            Self::FlatMap(op) => op.call_fn(event, rt_engine),
            Self::Reduce(op) => op.call_fn(event, rt_engine),
            Self::Empty => todo!(),
        }
    }
}

impl<S: state::StateManager> IOperator for KeyByOperator<S> {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i,
    {
        let mut new_events = BTreeMap::<TypedValue, KeyedDataEvent>::new();

        event
            .data
            .iter()
            .map(|entry| TypedValue::from(entry))
            .map(|typed_val| {
                (
                    rt_engine
                        .borrow_mut()
                        .call_one_arg(&typed_val)
                        .unwrap_or(TypedValue::Invalid),
                    typed_val,
                )
            })
            .map(|(key, val)| {
                let mut value_entry = Entry::default();

                value_entry.set_data_type(val.get_type());
                value_entry.value = val.get_data();
                (key, value_entry)
            })
            .for_each(|(key, value)| {
                if !new_events.contains_key(&key) {
                    let mut key_entry = Entry::default();
                    key_entry.set_data_type(key.get_type());
                    key_entry.value = key.get_data();

                    let mut event = KeyedDataEvent::default();
                    event.from_operator_id = self.operator_id;
                    event.key = Some(key_entry);
                    new_events.insert(key.clone(), event);
                }

                match new_events.get_mut(&key) {
                    Some(event) => {
                        event.data.push(value);
                    }
                    None => {}
                }
            });

        Ok(Vec::from_iter(
            new_events.iter().map(|entry| entry.1.clone()),
        ))
    }
}

impl<S: state::StateManager> IOperator for ReduceOperator<S> {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i,
    {
        let key =
            get_operator_state_key(self.operator_id, "reduce", event.get_key().value.as_slice());
        let state = self.state_manager.get_keyed_state(key.as_slice());

        let accum = if state.is_empty() {
            event
                .data
                .iter()
                .map(|entry| TypedValue::from(entry))
                .reduce(|prev, next| {
                    rt_engine
                        .borrow_mut()
                        .call_two_args((&prev, &next))
                        .unwrap_or(TypedValue::Invalid)
                })
                .unwrap_or(TypedValue::Invalid)
        } else {
            let accum = TypedValue::from_vec(&state);

            event.data.iter().fold(accum, |accum, entry| {
                let val = TypedValue::from_slice(&entry.value);
                rt_engine
                    .borrow_mut()
                    .call_two_args((&accum, &val))
                    .unwrap_or(TypedValue::Invalid)
            })
        };

        self.state_manager
            .set_key_state(key.as_slice(), accum.get_data().as_slice());

        let mut new_event = event.clone();
        let mut entry = Entry::default();
        entry.value = accum.get_data();
        entry.set_data_type(accum.get_type());
        new_event.data = vec![entry];
        Ok(vec![new_event])
    }
}

impl<S: state::StateManager> IOperator for FilterOperator<S> {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i,
    {
        let filtered = event
            .data
            .iter()
            .filter(|entry| {
                let val = TypedValue::from_slice(&entry.value);
                let result = rt_engine
                    .borrow_mut()
                    .call_one_arg(&val)
                    .unwrap_or(TypedValue::Boolean(false));
                match result {
                    TypedValue::Boolean(flag) => flag,
                    _ => false,
                }
            })
            .map(|entry| entry.clone());

        let mut new_event = event.clone();
        new_event.data = filtered.collect();
        Ok(vec![new_event])
    }
}

impl<S: state::StateManager> IOperator for FlatMapOperator<S> {
    fn call_fn<'p, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
    where
        'p: 'i,
    {
        let flat_map_value = event.data.iter().map(|entry| {
            let val = TypedValue::from_slice(&entry.value);
            let result = rt_engine
                .borrow_mut()
                .call_one_arg(&val)
                .unwrap_or(TypedValue::Array(vec![]));

            match result {
                TypedValue::Array(v) => v,
                _ => vec![],
            }
        });

        let mut new_event = event.clone();
        new_event.data = flat_map_value
            .map(|val| {
                val.iter()
                    .map(|value| {
                        let mut entry = Entry::default();
                        entry.set_data_type(value.get_type());
                        entry.value = value.get_data();

                        entry
                    })
                    .collect::<Vec<Entry>>()
            })
            .flatten()
            .collect();

        Ok(vec![new_event])
    }
}

macro_rules! define_operator {
    ($name: ident) => {
        pub(crate) struct $name<S>
        where
            S: state::StateManager,
        {
            state_manager: S,
            operator_id: NodeIdx,
        }
    };
}

macro_rules! new_operator {
    ($name:ident) => {
        impl<S> $name<S>
        where
            S: state::StateManager,
        {
            pub(crate) fn new(op_info: &OperatorInfo, state_manager: S) -> Self {
                let operator_id = op_info.operator_id;
                $name {
                    state_manager,
                    operator_id,
                }
            }
        }
    };
}

macro_rules! stateless_operator {
    ($name: ident) => {
        impl<S: state::StateManager> IOperator for $name<S> {
            fn call_fn<'p, 'i>(
                &self,
                event: &KeyedDataEvent,
                rt_engine: &RefCell<RuntimeEngine<'p, 'i>>,
            ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>
            where
                'p: 'i,
            {
                let mut new_event = event.clone();

                let value_entry_results = event
                    .data
                    .iter()
                    .map(|entry| TypedValue::from(entry))
                    .map(|typed_val| {
                        rt_engine
                            .borrow_mut()
                            .call_one_arg(&typed_val)
                            .unwrap_or(TypedValue::Invalid)
                    })
                    .map(|val| {
                        let mut entry = Entry::default();
                        entry.set_data_type(val.get_type());
                        entry.value = val.get_data();
                        entry
                    });
                new_event.data = Vec::from_iter(value_entry_results);
                new_event.from_operator_id = self.operator_id;
                Ok(vec![new_event])
            }
        }
    };
}

define_operator!(MapOperator);
new_operator!(MapOperator);
stateless_operator!(MapOperator);

define_operator!(FlatMapOperator);
new_operator!(FlatMapOperator);

define_operator!(FilterOperator);
new_operator!(FilterOperator);

define_operator!(KeyByOperator);
new_operator!(KeyByOperator);

define_operator!(ReduceOperator);
new_operator!(ReduceOperator);

fn get_operator_state_key(operator_id: NodeIdx, operator: &str, reference: &[u8]) -> Vec<u8> {
    let mut prefix = format!("{}-{}", operator, operator_id).as_bytes().to_vec();
    prefix.append(&mut reference.to_vec());
    prefix
}

#[derive(Clone)]
pub enum WindowAssignerImpl {
    Fixed(FixedKeyedWindowAssigner),
    Slide(SlideKeyedWindowAssigner),
    Session(SessionKeyedWindowAssigner),
    Empty,
}

impl WindowAssignerImpl {
    pub fn new(operator: &OperatorInfo) -> Self {
        let window = operator.get_window();
        let trigger = window.get_trigger();
        match window.get_value() {
            Some(window) => match window {
                proto::common::window::Value::Fixed(fixed) => {
                    Self::Fixed(FixedKeyedWindowAssigner::new(fixed, trigger))
                }
                proto::common::window::Value::Slide(slide) => {
                    Self::Slide(SlideKeyedWindowAssigner::new(slide, trigger))
                }
                proto::common::window::Value::Session(session) => {
                    Self::Session(SessionKeyedWindowAssigner::new(session, trigger))
                }
            },
            None => Self::Empty,
        }
    }

    pub(crate) fn assign_windows(&self, keyed_event: &KeyedDataEvent) -> Vec<KeyedWindow> {
        match self {
            WindowAssignerImpl::Fixed(assigner) => assigner.assign_windows(keyed_event),
            WindowAssignerImpl::Slide(assigner) => assigner.assign_windows(keyed_event),
            WindowAssignerImpl::Session(assigner) => assigner.assign_windows(keyed_event),
            WindowAssignerImpl::Empty => {
                vec![KeyedWindow {
                    inner: keyed_event.clone(),
                    event_time: keyed_event.event_time.as_ref().map(|timestamp| {
                        let naive_time = chrono::NaiveDateTime::from_timestamp(
                            timestamp.seconds,
                            timestamp.nanos as u32,
                        );
                        chrono::DateTime::from_utc(naive_time, chrono::Utc)
                    }),
                    window_start: None,
                    window_end: None,
                }]
            }
        }
    }

    pub(crate) fn group_by_key_and_window(
        &self,
        keyed_windows: &mut VecDeque<KeyedWindow>,
    ) -> Vec<KeyedWindow> {
        match self {
            WindowAssignerImpl::Fixed(assigner) => assigner.group_by_key_and_window(keyed_windows),
            WindowAssignerImpl::Slide(assigner) => assigner.group_by_key_and_window(keyed_windows),
            WindowAssignerImpl::Session(assigner) => {
                assigner.group_by_key_and_window(keyed_windows)
            }
            WindowAssignerImpl::Empty => keyed_windows.iter().map(|w| w.clone()).collect(),
        }
    }

    pub(crate) async fn trigger(&self) {}
}

#[tonic::async_trait]
pub trait KeyedWindowAssigner {
    fn assign_windows(&self, event: &KeyedDataEvent) -> Vec<KeyedWindow>;
    fn group_by_key_and_window(&self, windows: &mut VecDeque<KeyedWindow>) -> Vec<KeyedWindow> {
        // GroupByKey
        let ref mut group_by_key = lang::group_deque_as_btree_map(windows, |item| {
            item.inner.window = None;
            item.inner.get_key().value.clone()
        });

        // MergeWindows
        self.merge_windows(group_by_key);

        group_by_key
            .iter_mut()
            .map(|entry| {
                let mut results = vec![];
                // GroupAlsoByWindow
                while let Some(mut window) = entry.1.pop() {
                    if results.is_empty() {
                        results.push(window)
                    } else {
                        let mut filtered = results.iter_mut().filter(|w| {
                            w.window_start == window.window_start
                                && w.window_end == window.window_end
                        });

                        if filtered.next().is_none() {
                            results.push(window)
                        } else {
                            filtered.for_each(|w| w.inner.data.append(&mut window.inner.data));
                        }
                    }
                }

                results.iter_mut().for_each(|w| {
                    w.event_time = w.window_end.clone();
                });

                results
            })
            .reduce(|mut accum, mut current| {
                // ExpandToElements
                accum.append(&mut current);
                accum
            })
            .unwrap_or_default()
    }

    fn merge_windows(&self, group_by_key: &mut BTreeMap<Vec<u8>, Vec<KeyedWindow>>);

    async fn trigger(&self);
}

#[derive(Clone)]
pub struct FixedKeyedWindowAssigner {
    size: Duration,
    trigger: TriggerImpl,
}

impl FixedKeyedWindowAssigner {
    fn new(
        fixed: &proto::common::window::FixedWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> FixedKeyedWindowAssigner {
        let duration = fixed.get_size().to_duration();

        let trigger = trigger
            .and_then(|t| t.value.as_ref().map(|val| TriggerImpl::new(val)))
            .unwrap_or(TriggerImpl::default(&duration));

        FixedKeyedWindowAssigner {
            size: duration,
            trigger,
        }
    }
}

#[tonic::async_trait]
impl KeyedWindowAssigner for FixedKeyedWindowAssigner {
    fn assign_windows(&self, event: &KeyedDataEvent) -> Vec<KeyedWindow> {
        let event_time = event.get_event_time();
        vec![KeyedWindow {
            inner: event.clone(),
            event_time: event_time.clone(),
            window_start: event_time.clone(),
            window_end: event_time.and_then(|event_time| event_time.checked_add_signed(self.size)),
        }]
    }

    fn merge_windows(&self, group_by_key: &mut BTreeMap<Vec<u8>, Vec<KeyedWindow>>) {
        group_by_key.par_iter_mut().for_each(|entry| {
            let mut results = vec![];
            entry.1.sort_by(|a, b| b.window_start.cmp(&a.window_start));

            while let Some(mut window) = entry.1.pop() {
                if results.is_empty() {
                    results.push(window);
                } else {
                    let mut filter = results.iter_mut().filter(|w| {
                        (*w).window_end > window.window_start
                            && (*w).window_start <= window.window_start
                    });
                    let mut next = filter.next();
                    if next.is_none() {
                        results.push(window);
                    } else {
                        next.iter_mut().for_each(|w| {
                            w.inner.data.append(&mut window.inner.data);
                        })
                    }
                }
            }

            entry.1.append(&mut results);
        })
    }

    async fn trigger(&self) {
        self.trigger.emit()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct KeyedWindow {
    inner: KeyedDataEvent,
    event_time: Option<chrono::DateTime<chrono::Utc>>,
    window_start: Option<chrono::DateTime<chrono::Utc>>,
    window_end: Option<chrono::DateTime<chrono::Utc>>,
}
impl KeyedWindow {
    pub(crate) fn as_event(&mut self) -> &KeyedDataEvent {
        self.inner.event_time = self.event_time.map(|time| Timestamp {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos() as i32,
        });

        self.inner.window = self
            .window_end
            .iter()
            .zip(self.window_start.iter())
            .map(|pair| keyed_data_event::Window {
                start_time: Some(Timestamp {
                    seconds: pair.1.timestamp(),
                    nanos: pair.1.timestamp_subsec_nanos() as i32,
                }),
                end_time: Some(Timestamp {
                    seconds: pair.0.timestamp(),
                    nanos: pair.0.timestamp_subsec_nanos() as i32,
                }),
            })
            .next();

        &self.inner
    }
}

#[derive(Clone)]
pub struct SlideKeyedWindowAssigner {}

impl SlideKeyedWindowAssigner {
    fn new(
        slide: &proto::common::window::SlidingWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> SlideKeyedWindowAssigner {
        SlideKeyedWindowAssigner {}
    }
}

#[tonic::async_trait]
impl KeyedWindowAssigner for SlideKeyedWindowAssigner {
    fn assign_windows(&self, event: &KeyedDataEvent) -> Vec<KeyedWindow> {
        todo!()
    }

    fn merge_windows(&self, group_by_key: &mut BTreeMap<Vec<u8>, Vec<KeyedWindow>>) {}
    async fn trigger(&self) {}
}

#[derive(Clone)]
pub struct SessionKeyedWindowAssigner {}

impl SessionKeyedWindowAssigner {
    fn new(
        session: &proto::common::window::SessionWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> SessionKeyedWindowAssigner {
        SessionKeyedWindowAssigner {}
    }
}

#[tonic::async_trait]
impl KeyedWindowAssigner for SessionKeyedWindowAssigner {
    fn assign_windows(&self, event: &KeyedDataEvent) -> Vec<KeyedWindow> {
        todo!()
    }

    fn merge_windows(&self, group_by_key: &mut BTreeMap<Vec<u8>, Vec<KeyedWindow>>) {}
    async fn trigger(&self) {}
}

#[derive(Clone)]
pub enum TriggerImpl {
    Watermark {
        rx: crossbeam_channel::Receiver<()>,
        terminator: crossbeam_channel::Sender<()>,
    },
}
impl TriggerImpl {
    fn new(val: &proto::common::trigger::Value) -> Self {
        match val {
            proto::common::trigger::Value::Watermark(watermark) => {
                let trigger_time = watermark.get_trigger_time().to_duration();

                Self::new_watermark(&trigger_time)
            }
        }
    }

    fn default(duration: &Duration) -> Self {
        Self::new_watermark(&duration)
    }

    fn new_watermark(duration: &Duration) -> Self {
        let trigger_time = duration.to_std().unwrap_or(DETAULT_WATERMARK);
        let (tx, rx) = crossbeam_channel::bounded(1);

        let (terminator_tx, terminator_rx) = crossbeam_channel::bounded::<()>(1);

        tokio::spawn(async move {
            let sleep = tokio::time::sleep(trigger_time);
            tokio::pin!(sleep);
            loop {
                tokio::select! {
                    Ok(_) = async { terminator_rx.recv() } => {
                        drop(tx);
                        break;
                    },
                    _ = &mut sleep => {
                        let _ = tx.send(());
                    }
                }
            }
        });

        Self::Watermark {
            rx,
            terminator: terminator_tx,
        }
    }

    fn emit(&self) {
        match self {
            TriggerImpl::Watermark { rx, terminator } => match rx.recv() {
                Ok(_) => {}
                Err(_) => {
                    let _ = terminator.send(());
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};

    use prost_types::Timestamp;
    use proto::common::{
        filter, flat_map, key_by, mapper, operator_info::Details, reducer, window::FixedWindow,
        Entry, KeyedDataEvent, OperatorInfo, Time,
    };

    use crate::dataflow::KeyedWindow;

    use super::WindowAssignerImpl;

    fn get_opeartor_code(op_info: &OperatorInfo) -> String {
        match op_info.details.as_ref() {
            Some(info) => match info {
                Details::Mapper(func) => match &func.value {
                    Some(value) => match value {
                        mapper::Value::Func(func) => func.function.clone(),
                    },
                    None => "".to_string(),
                },
                Details::Filter(func) => match &func.value {
                    Some(value) => match value {
                        filter::Value::Func(func) => func.function.clone(),
                    },
                    None => "".to_string(),
                },
                Details::KeyBy(func) => match &func.value {
                    Some(value) => match value {
                        key_by::Value::Func(func) => func.function.clone(),
                    },
                    None => "".to_string(),
                },
                Details::Reducer(func) => match &func.value {
                    Some(value) => match value {
                        reducer::Value::Func(func) => func.function.clone(),
                    },
                    None => "".to_string(),
                },
                Details::FlatMap(func) => match &func.value {
                    Some(value) => match value {
                        flat_map::Value::Func(func) => func.function.clone(),
                    },
                    None => "".to_string(),
                },
                _ => "".to_string(),
            },
            None => "".to_string(),
        }
    }

    struct SetupGuard {}

    impl Drop for SetupGuard {
        fn drop(&mut self) {}
    }

    fn setup() -> SetupGuard {
        use crate::MOD_TEST_START;
        MOD_TEST_START.call_once(|| {
            v8::V8::set_flags_from_string(
                "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
              );
                  v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
                  v8::V8::initialize();
        });
        std::env::set_var("STATE_MANAGER", "MEM");

        SetupGuard {}
    }

    #[test]
    fn test_map_operator() {
        use super::MapOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{Mapper, OperatorInfo};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::Mapper(Mapper {
                value: Some(mapper::Value::Func(Func {
                    function: "function _operator_map_process(a) { return a+1 }".to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            "function _operator_map_process(a) { return a+1 }",
            &get_function_name(&op_info),
            isolated_scope,
        ));

        let operator = MapOperator::new(&op_info, state_manager);
        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let val = TypedValue::Number(1.0);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        event.data = vec![entry.clone(), entry.clone()];

        let result = operator.call_fn(&event, &rt_engine);
        assert!(result.is_ok());
        let new_events = result.expect("");
        assert_eq!(new_events.len(), 1);

        let mut entry = Entry::default();
        let val = TypedValue::Number(2.0);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();
        assert_eq!(
            new_events[0].data.as_slice(),
            &[entry.clone(), entry.clone()]
        );
    }

    #[test]
    fn test_filter_operator() {
        use super::FilterOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{Filter, OperatorInfo};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::Filter(Filter {
                value: Some(filter::Value::Func(Func {
                    function: "function _operator_filter_process(a) { return a === 1 }".to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            "function _operator_filter_process(a) { return a === 1 }",
            &get_function_name(&op_info),
            isolated_scope,
        ));

        let operator = FilterOperator::new(&op_info, state_manager);
        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let val = TypedValue::Number(1.0);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        let mut entry_1 = entry.clone();
        let val = TypedValue::Number(2.0);
        entry_1.set_data_type(val.get_type());
        entry_1.value = val.get_data();

        event.data = vec![entry.clone(), entry.clone(), entry_1];

        let result = operator.call_fn(&event, &rt_engine);

        {
            assert!(result.is_ok());
            let new_events = result.expect("");
            assert_eq!(new_events.len(), 1);

            assert_eq!(new_events[0].data, vec![entry.clone(), entry.clone()]);
        }
    }

    #[test]
    fn test_keyby_operator() {
        use std::collections::BTreeMap;

        use super::KeyByOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use common::collections::lang::any_match;
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{KeyBy, OperatorInfo};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::KeyBy(KeyBy {
                value: Some(key_by::Value::Func(Func {
                    function: "function _operator_keyBy_process(a) { return a.foo }".to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            &get_opeartor_code(&op_info),
            &get_function_name(&op_info),
            isolated_scope,
        ));
        let operator = KeyByOperator::new(&op_info, state_manager);

        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let mut val = BTreeMap::default();
        val.insert("foo".to_string(), TypedValue::String("bar".to_string()));
        val.insert("foo2".to_string(), TypedValue::String("bar".to_string()));

        let val = TypedValue::Object(val);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        let mut entry_1 = entry.clone();
        let mut val = BTreeMap::default();
        val.insert("foo".to_string(), TypedValue::String("bar1".to_string()));
        val.insert("foo2".to_string(), TypedValue::String("bar2".to_string()));
        let val = TypedValue::Object(val);

        entry_1.set_data_type(val.get_type());
        entry_1.value = val.get_data();

        event.data = vec![entry.clone(), entry.clone(), entry_1];

        let result = operator.call_fn(&event, &rt_engine);
        {
            assert!(result.is_ok());
            let events = result.unwrap();
            assert_eq!(events.len(), 2);
            assert!(any_match(&events, |event| {
                event.key.is_some()
                    && TypedValue::from(event.key.as_ref().unwrap())
                        == TypedValue::String("bar".to_string())
            }));
            assert!(any_match(&events, |event| {
                event.key.is_some()
                    && TypedValue::from(event.key.as_ref().unwrap())
                        == TypedValue::String("bar1".to_string())
            }));
        }
    }

    #[test]
    fn test_reduce_operator() {
        use super::ReduceOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use crate::{dataflow::get_operator_state_key, state::StateManager};
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{OperatorInfo, Reducer};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::Reducer(Reducer {
                value: Some(reducer::Value::Func(Func {
                    function:
                        "function _operator_reduce_process(accum, val) { return accum + val }"
                            .to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            "function _operator_reduce_process(accum, val) { return accum + val }",
            &get_function_name(&op_info),
            isolated_scope,
        ));

        let operator = ReduceOperator::new(&op_info, state_manager);
        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let val = TypedValue::Number(1.0);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        let mut entry_1 = entry.clone();
        let val = TypedValue::Number(2.0);
        entry_1.set_data_type(val.get_type());
        entry_1.value = val.get_data();

        event.data = vec![entry.clone(), entry.clone(), entry_1];
        let result = operator.call_fn(&event, &rt_engine);

        {
            assert!(result.is_ok());
            let new_events = result.expect("");
            assert_eq!(new_events.len(), 1);
            let mut entry = Entry::default();
            let val = TypedValue::Number(4.0);
            entry.set_data_type(val.get_type());
            entry.value = val.get_data();

            assert_eq!(new_events[0].data, vec![entry]);
            let state = operator
                .state_manager
                .get_keyed_state(&get_operator_state_key(
                    operator.operator_id,
                    "reduce",
                    new_events[0].get_key().value.as_slice(),
                ));
            assert_eq!(TypedValue::from_vec(&state), val);
        }
    }

    #[test]
    fn test_flatmap_operator_return_array() {
        use super::FlatMapOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{FlatMap, OperatorInfo};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::FlatMap(FlatMap {
                value: Some(flat_map::Value::Func(Func {
                    function: "function _operator_flatMap_process(v) { return [v, v, 2] }"
                        .to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            "function _operator_flatMap_process(v) { return [v, v, 2] }",
            &get_function_name(&op_info),
            isolated_scope,
        ));
        let operator = FlatMapOperator::new(&op_info, state_manager);

        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let val = TypedValue::Number(1.0);
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        event.data = vec![entry.clone()];
        let result = operator.call_fn(&event, &rt_engine);

        {
            assert!(result.is_ok());
            let new_events = result.expect("");
            assert_eq!(new_events.len(), 1);
            let mut entry_1 = Entry::default();
            let val = TypedValue::Number(1.0);
            entry_1.set_data_type(val.get_type());
            entry_1.value = val.get_data();

            let mut entry_2 = Entry::default();
            let val = TypedValue::Number(1.0);
            entry_2.set_data_type(val.get_type());
            entry_2.value = val.get_data();

            let mut entry_3 = Entry::default();
            let val = TypedValue::Number(2.0);
            entry_3.set_data_type(val.get_type());
            entry_3.value = val.get_data();

            assert_eq!(new_events[0].data, vec![entry_1, entry_2, entry_3]);
        }
    }

    #[test]
    fn test_flatmap_operator_string_split() {
        use super::FlatMapOperator;
        use crate::dataflow::get_function_name;
        use crate::dataflow::IOperator;
        use crate::state::MemoryStateManager;
        use crate::v8_runtime::RuntimeEngine;
        use common::types::TypedValue;
        use proto::common::Func;
        use proto::common::{Entry, KeyedDataEvent};
        use proto::common::{FlatMap, OperatorInfo};
        use std::cell::RefCell;

        let _setup_guard = setup();

        let op_info = OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(Details::FlatMap(FlatMap {
                value: Some(flat_map::Value::Func(Func {
                    function: "function _operator_flatMap_process(value) { return value.split(\" \").map(v => { return { t0: 1, t1: v }; }) }".to_string(),
                })),
            })),
        };

        let state_manager = MemoryStateManager::new();

        let isolate = &mut v8::Isolate::new(Default::default());
        let isolated_scope = &mut v8::HandleScope::new(isolate);
        let rt_engine = RefCell::new(RuntimeEngine::new(
            "function _operator_flatMap_process(value) { return value.split(\" \").map(v => { return { t0: 1, t1: v }; }) }",
            &get_function_name(&op_info),
            isolated_scope,
        ));
        let operator = FlatMapOperator::new(&op_info, state_manager);

        let mut event = KeyedDataEvent::default();
        event.from_operator_id = 0;

        let mut entry = Entry::default();
        let val = TypedValue::String("value value value".to_string());
        entry.set_data_type(val.get_type());
        entry.value = val.get_data();

        event.data = vec![entry.clone()];
        let result = operator.call_fn(&event, &rt_engine);

        {
            assert!(result.is_ok());
            let new_events = result.expect("");
            assert_eq!(new_events.len(), 1);
            let mut entry_1 = Entry::default();
            let val = TypedValue::Object(BTreeMap::from_iter(
                [
                    ("t1".to_string(), TypedValue::String("value".to_string())),
                    ("t0".to_string(), TypedValue::Number(1.0)),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            entry_1.set_data_type(val.get_type());
            entry_1.value = val.get_data();

            assert_eq!(
                new_events[0].data,
                vec![entry_1.clone(), entry_1.clone(), entry_1.clone()]
            );
        }
    }

    #[tokio::test]
    async fn test_fixed_window_assigner() {
        let fixed = FixedWindow {
            size: Some(Time {
                millis: 300,
                seconds: 0,
                minutes: 0,
                hours: 0,
            }),
        };

        let assigner =
            WindowAssignerImpl::Fixed(super::FixedKeyedWindowAssigner::new(&fixed, None));

        // test assign_windows()
        {
            let now = chrono::Utc::now();
            let event_time = Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            };
            let event = KeyedDataEvent {
                job_id: None,
                key: None,
                to_operator_id: 1,
                data: vec![],
                event_time: Some(event_time),
                process_time: None,
                from_operator_id: 0,
                window: None,
            };

            let windows = assigner.assign_windows(&event);

            assert_eq!(
                windows,
                vec![KeyedWindow {
                    inner: event,
                    event_time: Some(now.clone()),
                    window_start: Some(now.clone()),
                    window_end: now.checked_add_signed(chrono::Duration::milliseconds(300))
                }]
            );
        }

        // test group_by_key_and_window()
        {
            let now = chrono::Utc::now();

            // unordered event input
            let events = vec![
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![1],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![2],
                    }],
                    event_time: Some(Timestamp {
                        seconds: now.timestamp(),
                        nanos: now.timestamp_subsec_nanos() as i32,
                    }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![1],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![3],
                    }],
                    event_time: now
                        .checked_add_signed(chrono::Duration::milliseconds(35))
                        .map(|t| Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.timestamp_subsec_nanos() as i32,
                        }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![1],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![4],
                    }],
                    event_time: now
                        .checked_add_signed(chrono::Duration::milliseconds(10))
                        .map(|t| Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.timestamp_subsec_nanos() as i32,
                        }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![2],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![5],
                    }],
                    event_time: now
                        .checked_add_signed(chrono::Duration::milliseconds(20))
                        .map(|t| Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.timestamp_subsec_nanos() as i32,
                        }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![2],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![6],
                    }],
                    event_time: now
                        .checked_add_signed(chrono::Duration::milliseconds(350))
                        .map(|t| Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.timestamp_subsec_nanos() as i32,
                        }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
                KeyedDataEvent {
                    job_id: None,
                    key: Some(Entry {
                        data_type: 1,
                        value: vec![3],
                    }),
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![7],
                    }],
                    event_time: now
                        .checked_add_signed(chrono::Duration::milliseconds(250))
                        .map(|t| Timestamp {
                            seconds: t.timestamp(),
                            nanos: t.timestamp_subsec_nanos() as i32,
                        }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                },
            ];

            let windows = events
                .iter()
                .map(|event| assigner.assign_windows(event))
                .reduce(|mut accum, mut current| {
                    accum.append(&mut current);
                    accum
                });

            assert!(windows.is_some());
            let windows = windows.unwrap();

            let windows = assigner.group_by_key_and_window(&mut VecDeque::from(windows));

            {
                let mut key_1_filter = windows.iter().filter(|w| {
                    w.inner.key
                        == Some(Entry {
                            data_type: 1,
                            value: vec![1],
                        })
                });

                let next = key_1_filter.next();
                assert_eq!(
                    next,
                    Some(&KeyedWindow {
                        inner: KeyedDataEvent {
                            job_id: None,
                            key: Some(Entry {
                                data_type: 1,
                                value: vec![1]
                            }),
                            to_operator_id: 1,
                            data: vec![
                                Entry {
                                    data_type: 1,
                                    value: vec![2]
                                },
                                Entry {
                                    data_type: 1,
                                    value: vec![4]
                                },
                                Entry {
                                    data_type: 1,
                                    value: vec![3]
                                }
                            ],
                            event_time: Some(Timestamp {
                                seconds: now.timestamp(),
                                nanos: now.timestamp_subsec_nanos() as i32,
                            }),
                            process_time: None,
                            from_operator_id: 0,
                            window: None
                        },
                        event_time: now.checked_add_signed(chrono::Duration::milliseconds(300)),
                        window_start: Some(now.clone()),
                        window_end: now.checked_add_signed(chrono::Duration::milliseconds(300))
                    })
                )
            }

            {
                let mut key_3_filter = windows.iter().filter(|w| {
                    w.inner.key
                        == Some(Entry {
                            data_type: 1,
                            value: vec![3],
                        })
                });

                let next = key_3_filter.next();
                assert_eq!(
                    next,
                    Some(&KeyedWindow {
                        inner: KeyedDataEvent {
                            job_id: None,
                            key: Some(Entry {
                                data_type: 1,
                                value: vec![3]
                            }),
                            to_operator_id: 1,
                            data: vec![Entry {
                                data_type: 1,
                                value: vec![7]
                            }],
                            event_time: now
                                .checked_add_signed(chrono::Duration::milliseconds(250))
                                .map(|t| Timestamp {
                                    seconds: t.timestamp(),
                                    nanos: t.timestamp_subsec_nanos() as i32,
                                }),
                            process_time: None,
                            from_operator_id: 0,
                            window: None
                        },
                        event_time: now.checked_add_signed(chrono::Duration::milliseconds(550)),
                        window_start: now.checked_add_signed(chrono::Duration::milliseconds(250)),
                        window_end: now.checked_add_signed(chrono::Duration::milliseconds(550))
                    })
                )
            }
        }
    }
}

use std::{cell::RefCell, collections::BTreeMap};

use common::types::{NodeIdx, TypedValue};
use proto::common::{
    event::{Entry, KeyedDataEvent},
    stream::{
        OperatorInfo,
        OperatorInfo_oneof_details::{filter, flat_map, key_by, mapper, reducer},
    },
};
use protobuf::RepeatedField;

use crate::{
    state,
    v8_runtime::{wrap_value, RuntimeEngine},
};

pub enum Window {
    Sliding { size: i32, period: i32 },
}

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
        op_info: OperatorInfo,
        state_manager: S,
        isolated_scope: &'i mut v8::HandleScope<'s, ()>,
    ) -> Self {
        let (rt_engine, operator) = if op_info.clone().details.is_some() {
            let detail = op_info.clone().details.unwrap();
            match detail {
                mapper(map_func) => (
                    RefCell::new(RuntimeEngine::new(
                        map_func.get_func().get_function(),
                        get_function_name(op_info.clone()).as_str(),
                        isolated_scope,
                    )),
                    OperatorImpl::Map(MapOperator::new(&op_info, state_manager)),
                ),
                filter(filter_fn) => (
                    RefCell::new(RuntimeEngine::new(
                        filter_fn.get_func().get_function(),
                        get_function_name(op_info.clone()).as_str(),
                        isolated_scope,
                    )),
                    OperatorImpl::Filter(FilterOperator::new(&op_info, state_manager)),
                ),
                key_by(key_by_fn) => (
                    RefCell::new(RuntimeEngine::new(
                        key_by_fn.get_func().get_function(),
                        get_function_name(op_info.clone()).as_str(),
                        isolated_scope,
                    )),
                    OperatorImpl::KeyBy(KeyByOperator::new(&op_info, state_manager)),
                ),
                reducer(reduce_fn) => (
                    RefCell::new(RuntimeEngine::new(
                        reduce_fn.get_func().get_function(),
                        get_function_name(op_info.clone()).as_str(),
                        isolated_scope,
                    )),
                    OperatorImpl::Reduce(ReduceOperator::new(&op_info, state_manager)),
                ),
                flat_map(flat_map_fn) => (
                    RefCell::new(RuntimeEngine::new(
                        flat_map_fn.get_func().get_function(),
                        get_function_name(op_info.clone()).as_str(),
                        isolated_scope,
                    )),
                    OperatorImpl::FlatMap(FlatMapOperator::new(&op_info, state_manager)),
                ),
                _ => (
                    RefCell::new(RuntimeEngine::new("", "", isolated_scope)),
                    OperatorImpl::Empty,
                ),
            }
        } else {
            (
                RefCell::new(RuntimeEngine::new("", "", isolated_scope)),
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

pub struct RunnableTaskError {}

fn get_function_name(op_info: OperatorInfo) -> String {
    match op_info.clone().details {
        Some(info) => match info {
            mapper(_) => format!("_operator_{}_process", "map"),
            filter(_) => format!("_operator_{}_process", "filter"),
            key_by(_) => format!("_operator_{}_process", "keyBy"),
            reducer(_) => format!("_operator_{}_process", "reduce"),
            flat_map(_) => format!("_operator_{}_process", "flatMap"),
            _ => "".to_string(),
        },
        None => "".to_string(),
    }
}

trait IOperator {
    fn call_fn<'s, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError>;
}

enum OperatorImpl<S: state::StateManager> {
    Map(MapOperator<S>),
    Filter(FilterOperator<S>),
    KeyBy(KeyByOperator<S>),
    FlatMap(FlatMapOperator<S>),
    Reduce(ReduceOperator<S>),
    Empty,
}

impl<S: state::StateManager> IOperator for OperatorImpl<S> {
    fn call_fn<'s, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
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
    fn call_fn<'s, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
        let mut new_events = BTreeMap::<TypedValue, KeyedDataEvent>::new();
        let isolate = &mut v8::Isolate::new(Default::default());
        let handle_scope = &mut v8::HandleScope::new(isolate);

        event
            .get_data()
            .iter()
            .map(|entry| TypedValue::from(entry))
            .map(|typed_val| {
                (
                    rt_engine
                        .borrow_mut()
                        .call_fn(&[wrap_value(&typed_val, handle_scope)])
                        .unwrap_or(TypedValue::Invalid),
                    typed_val,
                )
            })
            .map(|(key, val)| {
                let mut value_entry = Entry::default();

                value_entry.set_data_type(val.get_type());
                value_entry.set_value(val.get_data());
                (key, value_entry)
            })
            .for_each(|(key, value)| {
                if !new_events.contains_key(&key) {
                    let mut key_entry = Entry::default();
                    key_entry.set_data_type(key.get_type());
                    key_entry.set_value(key.get_data());

                    let mut event = KeyedDataEvent::default();
                    event.set_from_operator_id(self.operator_id);
                    event.set_key(key_entry);
                    new_events.insert(key.clone(), event);
                }

                match new_events.get_mut(&key) {
                    Some(event) => {
                        let mut fields = RepeatedField::from_slice(event.get_data());
                        fields.push(value);
                        (*event).set_data(fields);
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
    fn call_fn<'s, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
        todo!()
    }
}

impl<S: state::StateManager> IOperator for FilterOperator<S> {
    fn call_fn<'s, 'i>(
        &self,
        event: &KeyedDataEvent,
        rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
    ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
        todo!()
    }
}

macro_rules! define_operator {
    ($name: ident) => {
        struct $name<S>
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
            fn new(op_info: &OperatorInfo, state_manager: S) -> Self {
                let operator_id = op_info.get_operator_id();
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
            fn call_fn<'s, 'i>(
                &self,
                event: &KeyedDataEvent,
                rt_engine: &RefCell<RuntimeEngine<'s, 'i>>,
            ) -> Result<Vec<KeyedDataEvent>, RunnableTaskError> {
                let mut new_event = event.clone();
                let isolate = &mut v8::Isolate::new(Default::default());
                let handle_scope = &mut v8::HandleScope::new(isolate);

                let value_entry_results = event
                    .get_data()
                    .iter()
                    .map(|entry| TypedValue::from(entry))
                    .map(|typed_val| {
                        rt_engine
                            .borrow_mut()
                            .call_fn(&[wrap_value(&typed_val, handle_scope)])
                            .unwrap_or(TypedValue::Invalid)
                    })
                    .map(|val| {
                        let mut entry = Entry::default();
                        entry.set_data_type(val.get_type());
                        entry.set_value(val.get_data());
                        entry
                    });
                new_event.set_data(RepeatedField::from_iter(value_entry_results));
                new_event.set_from_operator_id(self.operator_id);
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
stateless_operator!(FlatMapOperator);

define_operator!(FilterOperator);
new_operator!(FilterOperator);

define_operator!(KeyByOperator);
new_operator!(KeyByOperator);

define_operator!(ReduceOperator);
new_operator!(ReduceOperator);

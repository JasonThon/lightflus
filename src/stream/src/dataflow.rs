use std::cell::RefCell;

use common::types::TypedValue;
use proto::common::{
    event::{Entry, KeyedDataEvent},
    stream::{
        OperatorInfo,
        OperatorInfo_oneof_details::{filter, flat_map, key_by, mapper, reducer},
    },
};
use protobuf::RepeatedField;

use crate::{state, v8_runtime::RuntimeEngine};

pub enum Window {
    Sliding { size: i32, period: i32 },
}

pub struct TaskContainer<'s, 'i, S: state::StateManager>
where
    's: 'i,
{
    state_manager: S,
    op_info: OperatorInfo,
    rt_engine: RefCell<RuntimeEngine<'s, 'i>>,
}

impl<'s, 'i, S: state::StateManager> TaskContainer<'s, 'i, S>
where
    's: 'i,
{
    pub fn new(
        op_info: OperatorInfo,
        state_manager: S,
        isolated_scope: &'i mut v8::HandleScope<'s, ()>,
    ) -> Self {
        let rt_engine = if op_info.clone().details.is_some() {
            let detail = op_info.clone().details.unwrap();
            match detail {
                mapper(map_func) => RefCell::new(RuntimeEngine::new(
                    map_func.get_func().get_function(),
                    get_function_name(op_info.clone()).as_str(),
                    isolated_scope,
                )),
                filter(filter_fn) => RefCell::new(RuntimeEngine::new(
                    filter_fn.get_func().get_function(),
                    get_function_name(op_info.clone()).as_str(),
                    isolated_scope,
                )),
                key_by(key_by_fn) => RefCell::new(RuntimeEngine::new(
                    key_by_fn.get_func().get_function(),
                    get_function_name(op_info.clone()).as_str(),
                    isolated_scope,
                )),
                reducer(reduce_fn) => RefCell::new(RuntimeEngine::new(
                    reduce_fn.get_func().get_function(),
                    get_function_name(op_info.clone()).as_str(),
                    isolated_scope,
                )),
                flat_map(flat_map_fn) => RefCell::new(RuntimeEngine::new(
                    flat_map_fn.get_func().get_function(),
                    get_function_name(op_info.clone()).as_str(),
                    isolated_scope,
                )),
                _ => RefCell::new(RuntimeEngine::new("", "", isolated_scope)),
            }
        } else {
            RefCell::new(RuntimeEngine::new("", "", isolated_scope))
        };

        Self {
            state_manager,
            op_info: op_info.clone(),
            rt_engine,
        }
    }

    pub(crate) fn process(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<KeyedDataEvent, RunnableTaskError> {
        self.call_fn(event)
    }

    fn call_fn(&self, event: &KeyedDataEvent) -> Result<KeyedDataEvent, RunnableTaskError> {
        let results = event
            .get_data()
            .iter()
            .map(|entry| TypedValue::from(entry))
            .map(|typed_val| {
                self.rt_engine
                    .borrow_mut()
                    .call_fn(typed_val)
                    .unwrap_or(TypedValue::Invalid)
            })
            .map(|val| {
                let mut entry = Entry::default();
                entry.set_data_type(val.get_type());
                entry.set_value(val.get_data());
                entry
            });

        let mut new_event = event.clone();
        new_event.set_data(RepeatedField::from_iter(results));
        new_event.set_from_operator_id(self.op_info.operator_id);
        Ok(new_event)
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

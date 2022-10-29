use common::types::TypedValue;
use proto::common::stream::{Filter_oneof_value, Mapper};
use proto::common::{
    event::{Entry, KeyedDataEvent},
    stream::{
        Mapper_oneof_value, OperatorInfo,
        OperatorInfo_oneof_details::{filter, flat_map, key_by, mapper, reducer},
    },
};
use protobuf::RepeatedField;

use crate::{state, v8_runtime::V8RuntimeContext};

pub enum Window {
    Sliding { size: i32, period: i32 },
}

pub struct TaskContext<'s, S: state::StateManager> {
    state_manager: S,
    op_info: OperatorInfo,
    runtime_ctx: Option<V8RuntimeContext<'s>>,
}

impl<'s, S: state::StateManager> TaskContext<'s, S> {
    pub fn new(op_info: OperatorInfo, state_manager: S) -> Self {
        let Some(detail) = op_info.details;
        let runtime_ctx = match detail {
            mapper(map_func) => Some(V8RuntimeContext::new(map_func.get_func().get_function())),
            filter(filter_fn) => Some(V8RuntimeContext::new(filter_fn.get_func().get_function())),
            key_by(key_by_fn) => Some(V8RuntimeContext::new(key_by_fn.get_func().get_function())),
            reducer(reduce_fn) => Some(V8RuntimeContext::new(reduce_fn.get_func().get_function())),
            flat_map(flat_map_fn) => {
                Some(V8RuntimeContext::new(flat_map_fn.get_func().get_function()))
            }
            _ => None,
        };
        Self {
            state_manager,
            op_info,
            runtime_ctx,
        }
    }

    pub(crate) fn process(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<KeyedDataEvent, RunnableTaskError> {
        self.op_info
            .details
            .map(|detail| match detail {
                mapper(mapfn) => self.map_fn(event, mapfn),
                filter(filterfn) => match filterfn.value {
                    Some(value) => match value {
                        Filter_oneof_value::func(function) => Ok(KeyedDataEvent::default()),
                    },
                    None => Ok(KeyedDataEvent::default()),
                },
                key_by(_) => todo!(),
                reducer(_) => todo!(),
                _ => Ok(KeyedDataEvent::default()),
            })
            .unwrap_or(Ok(KeyedDataEvent::default()))
    }

    fn map_fn(
        &self,
        event: &KeyedDataEvent,
        mapper: Mapper,
    ) -> Result<KeyedDataEvent, RunnableTaskError> {
        match mapper.value {
            Some(value) => match value {
                Mapper_oneof_value::func(function) => {
                    let ref mut runtime_ctx = V8RuntimeContext::new(function.get_function());
                    let results = event
                        .get_data()
                        .iter()
                        .map(|entry| TypedValue::from(entry))
                        .map(|typed_val| {
                            self.runtime_ctx
                                .call_fn(self.get_map_fn_name(), typed_val)
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
            },
            None => Ok(KeyedDataEvent::default()),
        }
    }

    fn get_map_fn_name(&self) -> &str {
        format!("_Map_Fn_{}", self.op_info.get_operator_id()).as_str()
    }
}

pub struct RunnableTaskError {}

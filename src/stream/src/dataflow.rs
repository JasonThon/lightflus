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

use crate::{state, v8_runtime::RuntimeEngine};

const MAP_FUNC_NAME: &str = "_operator_map_process";

pub enum Window {
    Sliding { size: i32, period: i32 },
}

pub struct TaskContainer<'s, S: state::StateManager> {
    state_manager: S,
    op_info: OperatorInfo,
    runtime_ctx: Option<RuntimeEngine<'s>>,
}

impl<'s, S: state::StateManager> TaskContainer<'s, S> {
    pub fn new(op_info: OperatorInfo, state_manager: S) -> Self {
        Self {
            state_manager,
            op_info: op_info.clone(),
            runtime_ctx: op_info.details.and_then(|detail| match detail {
                mapper(map_func) => Some(RuntimeEngine::new(map_func.get_func().get_function())),
                filter(filter_fn) => Some(RuntimeEngine::new(filter_fn.get_func().get_function())),
                key_by(key_by_fn) => Some(RuntimeEngine::new(key_by_fn.get_func().get_function())),
                reducer(reduce_fn) => Some(RuntimeEngine::new(reduce_fn.get_func().get_function())),
                flat_map(flat_map_fn) => {
                    Some(RuntimeEngine::new(flat_map_fn.get_func().get_function()))
                }
                _ => None,
            }),
        }
    }

    pub(crate) fn process(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<KeyedDataEvent, RunnableTaskError> {
        self.op_info.clone()
            .details
            .map(|detail| match detail {
                mapper(mapfn) => self.map_fn(event, mapfn),
                filter(filterfn) => match filterfn.value {
                    Some(value) => match value {
                        Filter_oneof_value::func(function) => Ok(KeyedDataEvent::default()),
                    },
                    None => Ok(KeyedDataEvent::default()),
                },
                key_by(keybyfn) => todo!(),
                reducer(_) => todo!(),
                _ => Ok(KeyedDataEvent::default()),
            })
            .unwrap_or(Ok(KeyedDataEvent::default()))
    }

    fn map_fn(
        &self,
        event: &KeyedDataEvent,
        mapfn: Mapper,
    ) -> Result<KeyedDataEvent, RunnableTaskError> {
        match mapfn.value {
            Some(value) => match value {
                Mapper_oneof_value::func(function) => {
                    let results = event
                        .get_data()
                        .iter()
                        .map(|entry| TypedValue::from(entry))
                        .map(|typed_val| {
                            self.runtime_ctx
                                .unwrap()
                                .call_fn(MAP_FUNC_NAME, typed_val)
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
}

pub struct RunnableTaskError {}

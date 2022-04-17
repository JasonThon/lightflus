use std::{collections, hash, marker};
use crate::{event, state, types};
use crate::state::StateManager;
use crate::stream::window;
use crate::types::formula;

pub trait Pipeline<InputKey, InputValue, Output, State>: Sync + Send
    where InputKey: hash::Hash + Clone + Eq, InputValue: Clone {
    type Context;

    fn apply(&self, input: &window::KeyedWindow<InputKey, InputValue>, ctx: &mut Context<State>) -> Output;
}

pub struct FormulaOpEventPipeline {
    job_id: types::JobID,
    op: formula::FormulaOp,
    node_id: u64,
}

unsafe impl Send for FormulaOpEventPipeline {}

unsafe impl Sync for FormulaOpEventPipeline {}

impl Pipeline<u64, types::ActionValue, Vec<event::FormulaOpEvent>, types::ValueState> for FormulaOpEventPipeline {
    type Context = Context<types::ValueState>;

    fn apply(&self, input: &window::KeyedWindow<u64, types::ActionValue>, ctx: &mut Self::Context) -> Vec<event::FormulaOpEvent> {
        let ref row_idx = input.key;
        let calculator = Calculator {
            op: &self.op
        };

        let group = common::lists::group_hashmap(&input.values, |value| value.from.clone());
        group.iter()
            .map(|(from, actions)| {
                let last_value = *actions.last().unwrap();
                if last_value.action.is_value_update() {
                    (from, Some(last_value))
                } else {
                    (from, None)
                }
            })
            .for_each(|(from, option)| {
                calculator.incre_calc(from, option, ctx)
            });

        let typed_value: types::TypedValue = calculator.take();

        vec![
            event::FormulaOpEvent {
                row_idx: row_idx.clone(),
                job_id: self.job_id.clone(),
                data: typed_value.get_data(),
                from: self.node_id.clone(),
                action: types::ActionType::INSERT,
                event_time: std::time::SystemTime::now(),
                data_type: typed_value.get_type(),
            }
        ]
    }
}

impl FormulaOpEventPipeline {
    pub fn new(op: formula::FormulaOp, job_id: types::JobID, node_id: u64) -> FormulaOpEventPipeline {
        FormulaOpEventPipeline {
            job_id,
            op,
            node_id,
        }
    }
}

pub struct Context<State> {
    phantom: marker::PhantomData<State>,
    state: state::RocksStateManager<String, State>,
}

impl<State> Context<State> {
    pub fn new() -> Context<State> {
        Context {
            phantom: Default::default(),
            state: state::RocksStateManager::new(),
        }
    }
}

unsafe impl<State> Send for Context<State> {}

unsafe impl<State> Sync for Context<State> {}

impl<State> Drop for Context<State> {
    fn drop(&mut self) {
        log::debug!("clearing context....");
        todo!()
    }
}


pub struct Calculator<'a> {
    op: &'a formula::FormulaOp,
}

impl<'a> Calculator<'a> {
    pub fn incre_calc(&self, from: &'a u64, value_option: Option<&'a types::ActionValue>, ctx: &'a mut Context<types::ValueState>) {
        todo!()
    }
    pub fn take(&self) -> types::TypedValue {
        todo!()
    }
}
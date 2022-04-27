use std::{collections, hash, marker};
use crate::{err, event, types};
use crate::stream::state::StateManager;
use crate::stream::{state, window};
use crate::types::formula;

pub type Result<T> = std::result::Result<T, err::PipelineError>;

pub trait Pipeline<InputKey, InputValue, Output, State>: Sync + Send
    where InputKey: hash::Hash + Clone + Eq, InputValue: Clone, State: Clone {
    type Context;

    fn apply(&self, input: &window::KeyedWindow<InputKey, InputValue>, ctx: &Context<State>) -> Result<Output>;
}

pub struct FormulaOpEventPipeline {
    job_id: types::JobID,
    op: formula::FormulaOp,
    node_id: u64,
    state: state::RocksStateManager<String, types::ValueState>,
}

unsafe impl Send for FormulaOpEventPipeline {}

unsafe impl Sync for FormulaOpEventPipeline {}

impl Pipeline<u64, types::ActionValue, Vec<event::FormulaOpEvent>, types::ValueState> for FormulaOpEventPipeline {
    type Context = Context<types::ValueState>;

    fn apply(&self, input: &window::KeyedWindow<u64, types::ActionValue>, ctx: &Self::Context) -> Result<Vec<event::FormulaOpEvent>> {
        let ref row_idx = input.key;
        let calculator = Calculator {
            op: &self.op
        };

        let group = common::lists::group_hashmap(&input.values, |value| value.from.clone());
        group.iter()
            .map(|(from, actions)| {
                let last_value = *actions.last().unwrap();

                (from, last_value)
            })
            .for_each(|(_, value)| {
                calculator.recv_state(value, ctx)
            });

        calculator.calc()
            .map(|typed_value| vec![
                event::FormulaOpEvent {
                    row_idx: row_idx.clone(),
                    job_id: self.job_id.clone(),
                    data: typed_value.get_data(),
                    from: self.node_id.clone(),
                    action: types::ActionType::INSERT,
                    event_time: std::time::SystemTime::now(),
                    data_type: typed_value.get_type(),
                }
            ])
    }
}

impl FormulaOpEventPipeline {
    pub fn new(op: formula::FormulaOp, job_id: types::JobID, node_id: u64) -> FormulaOpEventPipeline {
        FormulaOpEventPipeline {
            job_id: job_id.clone(),
            op,
            node_id,
            state: state::RocksStateManager::new(job_id),
        }
    }
}

pub struct Context<State> where State: Clone {
    phantom: marker::PhantomData<State>,
}

impl<State> Context<State> where State: Clone {
    pub fn new() -> Context<State> {
        Context {
            phantom: Default::default(),
        }
    }
}

unsafe impl<State> Send for Context<State> where State: Clone {}

unsafe impl<State> Sync for Context<State> where State: Clone {}

impl<State> Drop for Context<State> where State: Clone {
    fn drop(&mut self) {
        log::debug!("clearing context....");
    }
}


pub struct Calculator<'a> {
    op: &'a formula::FormulaOp,
}

impl<'a> Calculator<'a> {
    pub fn recv_state(&self, value: &'a types::ActionValue, ctx: &'a Context<types::ValueState>) {
        todo!()
    }
    pub fn calc(&self) -> Result<types::TypedValue> {
        todo!()
    }
}
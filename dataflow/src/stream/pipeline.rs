use std::{collections, hash, marker};

use crate::{err, event, types};
use crate::stream::{state, window};
use crate::stream::state::StateManager;
use crate::types::{ActionValue, formula, RowIdx};

pub type Result<T> = std::result::Result<T, err::PipelineError>;

pub trait Pipeline<InputKey, InputValue, Output, State>: Sync + Send
    where InputKey: hash::Hash + Clone + Eq, InputValue: Clone, State: Clone {
    type Context;

    fn apply(&self, input: &window::KeyedWindow<InputKey, InputValue>, ctx: &Context<InputKey, State>) -> Result<Output>;
}

pub struct FormulaOpEventPipeline {
    job_id: types::JobID,
    op: formula::FormulaOp,
    node_id: u64,
    state: state::RocksStateManager<String, types::ValueState>,
}

unsafe impl Send for FormulaOpEventPipeline {}

unsafe impl Sync for FormulaOpEventPipeline {}

impl Pipeline<types::RowIdx, types::ActionValue, Vec<event::FormulaOpEvent>, types::ValueState> for FormulaOpEventPipeline {
    type Context = Context<types::RowIdx, types::ValueState>;

    fn apply(&self, input: &window::KeyedWindow<u64, types::ActionValue>, ctx: &Self::Context) -> Result<Vec<event::FormulaOpEvent>> {
        let ref row_idx = input.key;
        let calculator = Calculator {
            op: &self.op
        };
        let old_state = ctx.get_state(row_idx);

        let group = common::lists::group_hashmap(&input.values, |value| value.from.clone());
        group.iter()
            .map(|(from, actions)| {
                let last_value = *actions.last().unwrap();

                (from, last_value)
            })
            .for_each(|(_, value)| {
                calculator.recv_state(row_idx, value, ctx)
            });

        calculator.calc(ctx)
            .map(|typed_value| vec![
                event::FormulaOpEvent {
                    row_idx: row_idx.clone(),
                    job_id: self.job_id.clone(),
                    data: typed_value.get_data(),
                    old_data: old_state
                        .map(|state| state.value
                            .clone()
                            .map(|val| val.get_data())
                            .unwrap_or(vec![])
                        )
                        .unwrap_or(vec![]),
                    from: self.node_id.clone(),
                    action: types::ActionType::UPDATE,
                    event_time: std::time::SystemTime::now(),
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

pub struct Context<Key, State> where State: Clone, Key: Clone + hash::Hash {
    phantom: marker::PhantomData<State>,
    phantom_key: marker::PhantomData<Key>,
}

impl<Key, State> Context<Key, State> where State: Clone, Key: Clone + hash::Hash {
    pub fn new() -> Context<Key, State> {
        Context {
            phantom: Default::default(),
            phantom_key: Default::default(),
        }
    }

    pub fn get_state(&self, key: &Key) -> Option<&State> {
        todo!()
    }
}

unsafe impl<Key, State> Send for Context<Key, State> where State: Clone, Key: Clone + hash::Hash {}

unsafe impl<Key, State> Sync for Context<Key, State> where State: Clone, Key: Clone + hash::Hash {}

impl<Key, State> Drop for Context<Key, State> where State: Clone, Key: Clone + hash::Hash {
    fn drop(&mut self) {
        log::debug!("clearing context....");
    }
}


pub struct Calculator<'a> {
    op: &'a formula::FormulaOp,
}

impl<'a> Calculator<'a> {
    pub fn recv_state(&self, row_idx: &u64, value: &'a types::ActionValue, ctx: &'a Context<types::RowIdx, types::ValueState>) {
        todo!()
    }
    pub fn calc(&self, ctx: &Context<types::RowIdx, types::ValueState>) -> Result<types::TypedValue> {
        todo!()
    }
}
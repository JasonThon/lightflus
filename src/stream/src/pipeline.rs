use std::{collections, hash, marker};
use common::{err, event, types};
use crate::{state, window};
use crate::state::StateManager;
use common::types::{formula, FromBytes};
use common::types::formula::FormulaOp;

pub type Result<T> = std::result::Result<T, err::PipelineError>;

pub trait Pipeline<InputKey, InputValue, Output, State>: Sync + Send
    where InputKey: hash::Hash + Clone + Eq + Ord, InputValue: Clone, State: Clone {
    type Context;

    fn apply(&self, input: &window::KeyedWindow<InputKey, InputValue>, ctx: &Context<InputKey, State>) -> Result<Output>;

    fn create_context(&self) -> Context<InputKey, State>;
}


pub struct Context<Key, State> where State: Clone, Key: Clone + hash::Hash + Eq + Ord {
    phantom: marker::PhantomData<State>,
    phantom_key: marker::PhantomData<Key>,
    job_id: types::JobID,
    values: common::collections::ConcurrentCache<Key, State>,
}

impl<Key, State> Context<Key, State> where State: Clone, Key: Clone + hash::Hash + Eq + Ord {
    pub fn new(job_id: types::JobID) -> Context<Key, State> {
        use std::default::Default;
        Context {
            phantom: Default::default(),
            phantom_key: Default::default(),
            job_id,
            values: Default::default(),
        }
    }

    pub fn get_state(&self, key: &Key) -> Option<State> {
        self.values.get(key)
    }

    pub fn update(&self, key: Key, state: State) -> Option<State> {
        self.values.put(key, state)
    }
}

unsafe impl<Key, State> Send for Context<Key, State> where State: Clone, Key: Clone + hash::Hash + Eq + Ord {}

unsafe impl<Key, State> Sync for Context<Key, State> where State: Clone, Key: Clone + hash::Hash + Eq + Ord {}


pub struct FormulaOpEventPipeline {
    job_id: types::JobID,
    op: formula::FormulaOp,
    node_id: types::NodeIdx,
    upstreams: Vec<types::NodeIdx>,
    state: state::RocksStateManager<String, types::ValueState>,
}

unsafe impl Send for FormulaOpEventPipeline {}

unsafe impl Sync for FormulaOpEventPipeline {}

impl Pipeline<types::RowIdx, types::ActionValue, Vec<event::FormulaOpEvent>, types::FormulaState> for FormulaOpEventPipeline {
    type Context = Context<types::RowIdx, types::FormulaState>;

    fn apply(&self, input: &window::KeyedWindow<u64, types::ActionValue>, ctx: &Self::Context) -> Result<Vec<event::FormulaOpEvent>> {
        let ref row_idx = input.key;
        let mut changed_row_idx = row_idx.clone();

        let option = ctx.get_state(row_idx);
        let group = common::lists::group_btree_map(&input.values, |value| value.from.clone());
        let ref mut node_values = group.iter()
            .map(|(from, actions)| {
                let last_value = *actions.last().unwrap();

                (*from, last_value.clone())
            })
            .collect::<collections::HashMap<types::NodeIdx, types::ActionValue>>();
        option.iter()
            .for_each(|v| v.node_states
                .iter()
                .for_each(|s|
                    if !node_values.contains_key(&s.node_idx) {
                        node_values.insert(s.node_idx, types::ActionValue {
                            action: types::ActionType::INSERT,
                            value: s.value.clone(),
                            from: s.node_idx,
                        });
                    })
            );

        let mut final_value = types::TypedValue::Invalid;

        match &self.op {
            FormulaOp::Reference { .. } => return Ok(vec![event::FormulaOpEvent {
                row_idx: *row_idx,
                job_id: self.job_id.clone(),
                data: node_values.get(&self.node_id)
                    .map(|v| (*v).value.get_data())
                    .unwrap_or_else(|| vec![]),
                old_data: vec![],
                from: self.node_id,
                action: node_values.get(&self.node_id)
                    .map(|v| (*v).action.clone())
                    .unwrap_or_else(|| types::ActionType::INVALID),
                event_time: input.timestamp,
            }]),
            FormulaOp::Add { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Add,
                );
            }
            FormulaOp::Sum => {
                final_value = sum_func(&self.upstreams[0], node_values, ctx);
                changed_row_idx = 0;
            }
            FormulaOp::Sumif => {
                final_value = condition_func(
                    &self.upstreams[0],
                    &self.upstreams[1],
                    node_values,
                    ctx,
                    sum_func,
                );
                changed_row_idx = 0;
            }
            FormulaOp::Countif => {
                final_value = condition_func(
                    &self.upstreams[0],
                    &self.upstreams[1],
                    node_values,
                    ctx,
                    count_func,
                );
                changed_row_idx = 0;
            }
            FormulaOp::Count => {
                final_value = count_func(&self.upstreams[0], node_values, ctx);
                changed_row_idx = 0;
            }
            FormulaOp::Avg => {
                final_value = avg_func(&self.upstreams[0], node_values, ctx);
                changed_row_idx = 0;
            }
            FormulaOp::Group => {}
            FormulaOp::Groupif => {}
            FormulaOp::Max => {
                final_value = max_func(&self.upstreams[0], node_values, ctx);
                changed_row_idx = 0;
            }
            FormulaOp::Maxif => {
                final_value = condition_func(
                    &self.upstreams[0],
                    &self.upstreams[1],
                    node_values,
                    ctx,
                    max_func,
                );
                changed_row_idx = 0;
            }
            FormulaOp::Min => {
                final_value = min_func(&self.upstreams[0], node_values, ctx);
                changed_row_idx = 0;
            }
            FormulaOp::Minif => {
                final_value = condition_func(
                    &self.upstreams[0],
                    &self.upstreams[1],
                    node_values,
                    ctx,
                    min_func,
                );
                changed_row_idx = 0;
            }
            FormulaOp::Xlookup => {}
            FormulaOp::Sub { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Sub,
                );
            }
            FormulaOp::Mul { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Mul,
                );
            }
            FormulaOp::Div { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Div,
                );
            }
            FormulaOp::Eq { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Eq,
                );
            }
            FormulaOp::Neq { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Neq,
                );
            }
            FormulaOp::Lt { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Lt,
                );
            }
            FormulaOp::Gt { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Gt,
                );
            }
            FormulaOp::Lte { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Lte,
                );
            }
            FormulaOp::Gte { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Gte,
                );
            }
            FormulaOp::And { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::And,
                );
            }
            FormulaOp::Or { values } => {
                final_value = dual_func(
                    values,
                    node_values,
                    self.upstreams.get(0),
                    self.upstreams.get(1),
                    DualOp::Or,
                );
            }
        }

        let _ = ctx.update(changed_row_idx, types::FormulaState {
            value: final_value.get_data(),
            node_states: node_values.values()
                .map(|v| types::ValueState {
                    value: v.value.clone(),
                    node_idx: v.from,
                })
                .collect(),
        });

        Ok(vec![event::FormulaOpEvent {
            row_idx: changed_row_idx,
            job_id: self.job_id.clone(),
            data: final_value.get_data(),
            old_data: option
                .as_ref()
                .map(|v| v.value.clone())
                .unwrap_or_else(|| vec![]),
            from: self.node_id,
            action: option
                .as_ref()
                .map(|_| types::ActionType::UPDATE)
                .unwrap_or(types::ActionType::INSERT),
            event_time: std::time::SystemTime::now(),
        }])
    }

    fn create_context(&self) -> Context<types::RowIdx, types::FormulaState> {
        Context::new(self.job_id.clone())
    }
}

impl FormulaOpEventPipeline {
    pub fn new(op: formula::FormulaOp,
               job_id: types::JobID,
               node_id: types::NodeIdx,
               upstreams: Vec<types::NodeIdx>) -> FormulaOpEventPipeline {
        FormulaOpEventPipeline {
            job_id: job_id.clone(),
            op,
            node_id,
            upstreams,
            state: state::RocksStateManager::new(job_id),
        }
    }
}

fn sum_func(ref_node: &types::NodeIdx,
            node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
            ctx: &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue {
    let sum_value = node_values.get(ref_node)
        .map(|v| {
            node_values.get(&v.from)
                .map(|sv| v.value.clone() + sv.value.clone())
                .unwrap_or_else(|| v.value.clone())
        })
        .map(|diff| ctx.get_state(&0)
            .map(|s| types::TypedValue::from(&s.value) + diff.clone())
            .unwrap_or_else(|| diff.clone()))
        .unwrap_or_else(|| types::TypedValue::Int(0));

    sum_value
}

fn count_func(ref_node: &types::NodeIdx,
              node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
              ctx: &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue {
    let option = ctx.get_state(&0);
    let diff = node_values.get(ref_node)
        .map(|v| match &v.value {
            types::TypedValue::Double(_) => types::TypedValue::Int(1),
            types::TypedValue::Float(_) => types::TypedValue::Int(1),
            types::TypedValue::Int(_) => types::TypedValue::Int(1),
            types::TypedValue::Long(_) => types::TypedValue::Int(1),
            _ => types::TypedValue::Int(0),
        })
        .unwrap_or(types::TypedValue::Int(0));

    option.map(|fs| types::TypedValue::from(&fs.value) + diff.clone())
        .unwrap_or(diff.clone())
}

fn avg_func(ref_node: &types::NodeIdx,
            node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
            ctx: &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue {
    todo!()
}

fn max_func(ref_node: &types::NodeIdx,
            node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
            ctx: &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue {
    todo!()
}

fn min_func(ref_node: &types::NodeIdx,
            node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
            ctx: &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue {
    todo!()
}

fn condition_func<F: Fn(&types::NodeIdx,
    &collections::HashMap<types::NodeIdx, types::ActionValue>,
    &Context<types::RowIdx, types::FormulaState>) -> types::TypedValue>(
    ref_node: &types::NodeIdx,
    condition_node: &types::NodeIdx,
    node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
    ctx: &Context<types::RowIdx, types::FormulaState>, f: F) -> types::TypedValue {
    let condition = node_values
        .get(condition_node)
        .map(|v| match v.value {
            types::TypedValue::Boolean(value) => value,
            _ => false
        })
        .unwrap_or(false);

    if condition { f(ref_node, node_values, ctx) } else { types::TypedValue::Invalid }
}

fn dual_func(values: &Vec<types::formula::ValueOp>,
             node_values: &collections::HashMap<types::NodeIdx, types::ActionValue>,
             left: Option<&types::NodeIdx>,
             right: Option<&types::NodeIdx>,
             op_type: DualOp) -> types::TypedValue {
    let left_node = left
        .map(|idx| node_values.get(idx))
        .unwrap_or(None);
    let right_node = right
        .map(|idx| node_values.get(idx))
        .unwrap_or(None);
    let value = values.get(0);
    let mut result = types::TypedValue::Invalid;
    match value {
        // TODO: refactor by macro
        None => {
            if left_node.is_some() & right_node.is_some() {
                result = match &op_type {
                    DualOp::Sub => left_node.unwrap().value.clone() - right_node.unwrap().value.clone(),
                    DualOp::Add => left_node.unwrap().value.clone() + right_node.unwrap().value.clone(),
                    DualOp::Mul => left_node.unwrap().value.clone() * right_node.unwrap().value.clone(),
                    DualOp::Div => left_node.unwrap().value.clone() / right_node.unwrap().value.clone(),
                    DualOp::Eq => types::TypedValue::Boolean(left_node.unwrap().value.clone() == right_node.unwrap().value.clone()),
                    DualOp::Neq => types::TypedValue::Boolean(left_node.unwrap().value.clone() != right_node.unwrap().value.clone()),
                    DualOp::Lt => types::TypedValue::Boolean(left_node.unwrap().value.clone() < right_node.unwrap().value.clone()),
                    DualOp::Lte => types::TypedValue::Boolean(left_node.unwrap().value.clone() <= right_node.unwrap().value.clone()),
                    DualOp::Gt => types::TypedValue::Boolean(left_node.unwrap().value.clone() > right_node.unwrap().value.clone()),
                    DualOp::Gte => types::TypedValue::Boolean(left_node.unwrap().value.clone() >= right_node.unwrap().value.clone()),
                    DualOp::And => left_node.unwrap().value.clone() & right_node.unwrap().value.clone(),
                    DualOp::Or => left_node.unwrap().value.clone() | right_node.unwrap().value.clone(),
                }
            } else if left_node.is_none() & right_node.is_some() {
                result = match &op_type {
                    DualOp::Sub => types::TypedValue::Int(-1) * right_node.unwrap().value.clone(),
                    DualOp::Add => right_node.unwrap().value.clone(),
                    DualOp::Mul => right_node.unwrap().value.clone(),
                    DualOp::Div => types::TypedValue::Int(1) / right_node.unwrap().value.clone(),
                    DualOp::Eq => types::TypedValue::Boolean(false),
                    DualOp::Neq => types::TypedValue::Boolean(false),
                    DualOp::Lt => types::TypedValue::Boolean(false),
                    DualOp::Lte => types::TypedValue::Boolean(false),
                    DualOp::Gt => types::TypedValue::Boolean(false),
                    DualOp::Gte => types::TypedValue::Boolean(false),
                    DualOp::And => types::TypedValue::Boolean(false),
                    DualOp::Or => types::TypedValue::Boolean(false),
                }
            } else if left_node.is_some() && right_node.is_none() {
                result = left_node.unwrap().value.clone();
            }
        }
        // TODO: refactor by macro
        Some(op) => {
            let value_op_val = types::TypedValue::from(&op.value);
            if left_node.is_some() {
                let left_value = left_node.unwrap().value.clone();
                if op.node_id < *left.unwrap() {
                    result = match &op_type {
                        DualOp::Sub => value_op_val.clone() - left_value,
                        DualOp::Add => value_op_val.clone() + left_value,
                        DualOp::Mul => value_op_val.clone() * left_value,
                        DualOp::Div => value_op_val.clone() / left_value,
                        DualOp::Eq => types::TypedValue::Boolean(value_op_val.clone() == left_value),
                        DualOp::Neq => types::TypedValue::Boolean(value_op_val.clone() != left_value),
                        DualOp::Lt => types::TypedValue::Boolean(value_op_val.clone() < left_value),
                        DualOp::Lte => types::TypedValue::Boolean(value_op_val.clone() <= left_value),
                        DualOp::Gt => types::TypedValue::Boolean(value_op_val.clone() > left_value),
                        DualOp::Gte => types::TypedValue::Boolean(value_op_val.clone() >= left_value),
                        DualOp::And => value_op_val.clone() & left_value,
                        DualOp::Or => value_op_val.clone() | left_value,
                    }
                } else {
                    result = match &op_type {
                        DualOp::Sub => left_value - value_op_val.clone(),
                        DualOp::Add => left_value + value_op_val.clone(),
                        DualOp::Mul => left_value * value_op_val.clone(),
                        DualOp::Div => left_value / value_op_val.clone(),
                        DualOp::Eq => types::TypedValue::Boolean(left_value == value_op_val.clone()),
                        DualOp::Neq => types::TypedValue::Boolean(left_value != value_op_val.clone()),
                        DualOp::Lt => types::TypedValue::Boolean(left_value < value_op_val.clone()),
                        DualOp::Lte => types::TypedValue::Boolean(left_value <= value_op_val.clone()),
                        DualOp::Gt => types::TypedValue::Boolean(left_value > value_op_val.clone()),
                        DualOp::Gte => types::TypedValue::Boolean(left_value >= value_op_val.clone()),
                        DualOp::And => value_op_val.clone() & left_value,
                        DualOp::Or => value_op_val.clone() | left_value,
                    }
                }
            }
        }
    };

    result
}

enum DualOp {
    Sub,
    Add,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}
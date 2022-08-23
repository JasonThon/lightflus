use common::types::ExecutorId;
use proto::common::stream::{OperatorType, StreamConfig};

pub enum Window {
    Sliding { size: i32, period: i32 },
}

pub struct RunnableOperator {
    op_type: OperatorType,
    config: StreamConfig,
    upstreams: Vec<ExecutorId>,
    
}

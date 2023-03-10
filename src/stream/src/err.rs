use std::fmt;

use common::{
    err::{KafkaException, RedisException, TaskWorkerError},
    event::KafkaEventError,
    types::NodeIdx,
};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidMessageType,
    MessageSendFailed,
    KafkaMessageSendFailed,
    SqlExecutionFailed,
    EventSentToRemoteFailed,
    RedisSinkFailed,
}

#[derive(Clone, Debug)]
pub struct SinkException {
    pub kind: ErrorKind,
    pub msg: String,
}

impl SinkException {
    pub fn into_task_worker_error(&self) -> TaskWorkerError {
        TaskWorkerError::EventSendFailure(format!("{:?}", self))
    }
}

impl From<KafkaException> for SinkException {
    fn from(err: KafkaException) -> Self {
        Self {
            kind: ErrorKind::KafkaMessageSendFailed,
            msg: format!("message detail: {}", err),
        }
    }
}

impl From<sqlx::Error> for SinkException {
    fn from(err: sqlx::Error) -> Self {
        Self {
            kind: ErrorKind::SqlExecutionFailed,
            msg: format!("{}", err),
        }
    }
}

impl From<&mut sqlx::Error> for SinkException {
    fn from(err: &mut sqlx::Error) -> Self {
        Self {
            kind: ErrorKind::SqlExecutionFailed,
            msg: format!("{}", err),
        }
    }
}

impl From<&mut RedisException> for SinkException {
    fn from(err: &mut RedisException) -> Self {
        Self {
            kind: ErrorKind::RedisSinkFailed,
            msg: format!("{:?}", err),
        }
    }
}

impl From<RedisException> for SinkException {
    fn from(err: RedisException) -> Self {
        Self {
            kind: ErrorKind::RedisSinkFailed,
            msg: format!("{:?}", err),
        }
    }
}

impl From<KafkaEventError> for SinkException {
    fn from(err: KafkaEventError) -> Self {
        Self {
            kind: ErrorKind::KafkaMessageSendFailed,
            msg: format!("{:?}", err),
        }
    }
}

impl From<&mut tonic::transport::Error> for SinkException {
    fn from(err: &mut tonic::transport::Error) -> Self {
        Self {
            kind: ErrorKind::EventSentToRemoteFailed,
            msg: err.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BatchSinkException {
    pub err: SinkException,
    pub event_id: u64,
}

#[derive(Debug)]
pub enum ExecutionError {
    OperatorUnimplemented(NodeIdx),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperatorUnimplemented(operator_id) => {
                f.write_str(format!("operator {} does not implement", operator_id).as_str())
            }
        }
    }
}

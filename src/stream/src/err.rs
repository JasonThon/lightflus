use std::fmt::{self, Display};

use common::{
    err::{KafkaException, RedisException},
    event::KafkaEventError,
    types::NodeIdx,
};

use crate::edge::OutEdgeError;

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

impl Display for SinkException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "sink to external sinker failed: [kind: {:?}], [message: {}]",
            self.kind, self.msg
        ))
    }
}

#[derive(Clone, Debug)]
pub struct BatchSinkException {
    pub err: SinkException,
    pub event_id: u64,
}

impl Display for BatchSinkException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "batchly sink events failed: [event_id: {}],[details: {}]",
            self.event_id, self.err
        ))
    }
}

impl From<&mut RedisException> for BatchSinkException {
    fn from(err: &mut RedisException) -> Self {
        Self {
            err: SinkException::from(err),
            event_id: 0,
        }
    }
}

impl From<RedisException> for BatchSinkException {
    fn from(err: RedisException) -> Self {
        Self {
            err: SinkException::from(err),
            event_id: 0,
        }
    }
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

#[derive(Debug)]
pub enum TaskError {
    OutEdgeError(OutEdgeError),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::OutEdgeError(err) => f.write_fmt(format_args!("out edge error [{}]", err)),
        }
    }
}

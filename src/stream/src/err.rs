use common::{
    err::{KafkaException, RedisException},
    event::{KafkaEventError, SinkableMessageImpl},
};
use crossbeam_channel::SendError;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidMessageType,
    MessageSendFailed,
    KafkaMessageSendFailed,
    SqlExecutionFailed,
    RemoteSinkFailed,
    RedisSinkFailed,
}

#[derive(Clone, Debug)]
pub struct SinkException {
    pub kind: ErrorKind,
    pub msg: String,
}

impl From<SendError<SinkableMessageImpl>> for SinkException {
    fn from(err: SendError<SinkableMessageImpl>) -> Self {
        Self {
            kind: ErrorKind::MessageSendFailed,
            msg: format!("message {:?} send to channel failed", err.0),
        }
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
        todo!()
    }
}

#[derive(Debug)]
pub struct RunnableTaskError {}

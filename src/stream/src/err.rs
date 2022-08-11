use tokio::sync::mpsc::error::SendError;
use crate::actor::SinkableMessageImpl;

#[derive(Debug, Clone)]
pub struct PipelineError {
    pub kind: ErrorKind,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidMessageType
}

#[derive(Clone, Debug)]
pub struct SinkException {
    kind: ErrorKind,
    msg: String,
}

impl From<SendError<SinkableMessageImpl>> for SinkException {
    fn from(_: SendError<SinkableMessageImpl>) -> Self {
        todo!()
    }
}

impl SinkException {
    pub(crate) fn invalid_message_type() -> Self {
        Self {
            kind: ErrorKind::InvalidMessageType,
            msg: "invalid message type".to_string(),
        }
    }
}

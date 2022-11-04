use crossbeam_channel::SendError;

use crate::actor::SinkableMessageImpl;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidMessageType,
    MessageSendFailed,
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

impl SinkException {
    pub(crate) fn invalid_message_type() -> Self {
        Self {
            kind: ErrorKind::InvalidMessageType,
            msg: "invalid message type".to_string(),
        }
    }
}

use std::io;

use tokio::sync::mpsc;
use crate::types;
use grpcio;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ApiError {
    pub code: u16,
    pub msg: String,
}

pub trait Error {
    fn to_string(&self) -> String {
        serde_json::to_string(
            &ApiError {
                code: self.code(),
                msg: format!("Error Kind: {:?}. Message: {}", self.kind(), self.msg()),
            })
            .unwrap()
    }

    fn kind(&self) -> ErrorKind;

    fn msg(&self) -> String;

    fn code(&self) -> u16;
}


#[derive(Clone, Debug)]
pub struct CommonException {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    NoAvailableWorker,
    ActorSendError,
    NodesRemoveFailed,
    InvalidJobGraph,
    SendGraphEventFailed,
    IllegalConnectionType,
    Timeout,
    InvalidEndpoint,
    ConnectionRefused,
    ConnectionAborted,
    NetworkDown,
    UnexpectedEof,
    NotConnected,
    NotFound,
    Unknown,
    MongoError(mongodb::error::ErrorKind),
    GrpcError,
    InvalidJson,
}

impl From<grpcio::Error> for CommonException {
    fn from(err: grpcio::Error) -> Self {
        Self {
            kind: ErrorKind::GrpcError,
            message: format!("{:?}", err),
        }
    }
}

impl From<mongodb::error::Error> for CommonException {
    fn from(err: mongodb::error::Error) -> Self {
        match err.kind.as_ref() {
            mongodb::error::ErrorKind::InvalidArgument {
                message, ..
            } => CommonException::new(
                ErrorKind::MongoError(err.kind.as_ref().clone()),
                message.as_str(),
            ),
            mongodb::error::ErrorKind::Authentication {
                message, ..
            } => CommonException::new(
                ErrorKind::MongoError(err.kind.as_ref().clone()),
                message.as_str(),
            ),
            mongodb::error::ErrorKind::BsonDeserialization(_) =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "bson fail to deserialize",
                ),
            mongodb::error::ErrorKind::BsonSerialization(_) =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "bson fail to serialize",
                ),
            mongodb::error::ErrorKind::BulkWrite(_) =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "bulk write failed",
                ),
            mongodb::error::ErrorKind::Command(_) =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "command error",
                ),
            mongodb::error::ErrorKind::DnsResolve { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::Internal { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::Io(_) =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "bson fail to serialize",
                ),
            mongodb::error::ErrorKind::ConnectionPoolCleared { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::InvalidResponse { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::ServerSelection { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::SessionsNotSupported =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    "session not support",
                ),
            mongodb::error::ErrorKind::InvalidTlsConfig { message, .. } =>
                CommonException::new(
                    ErrorKind::MongoError(err.kind.as_ref().clone()),
                    message.as_str(),
                ),
            mongodb::error::ErrorKind::Write(_) => CommonException::new(
                ErrorKind::MongoError(err.kind.as_ref().clone()),
                "write failed",
            ),
            mongodb::error::ErrorKind::Transaction { message, .. } => CommonException::new(
                ErrorKind::MongoError(err.kind.as_ref().clone()),
                message.as_str(),
            ),
            mongodb::error::ErrorKind::IncompatibleServer { message, .. } => CommonException::new(
                ErrorKind::MongoError(err.kind.as_ref().clone()),
                message.as_str(),
            ),
            _ => CommonException::new(ErrorKind::Unknown, "unknown")
        }
    }
}

impl From<std::io::Error> for CommonException {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::TimedOut =>
                CommonException::new(ErrorKind::Timeout, "request timeout"),
            io::ErrorKind::AddrNotAvailable =>
                CommonException::new(ErrorKind::InvalidEndpoint, "endpoint address is invalid"),
            io::ErrorKind::ConnectionRefused =>
                CommonException::new(ErrorKind::ConnectionRefused, "connection refused"),
            io::ErrorKind::ConnectionAborted =>
                CommonException::new(ErrorKind::ConnectionAborted, "connection abort"),
            io::ErrorKind::UnexpectedEof =>
                CommonException::new(ErrorKind::UnexpectedEof, "unexpected eof"),
            io::ErrorKind::NotConnected =>
                CommonException::new(ErrorKind::NotConnected, "not connected"),
            io::ErrorKind::NotFound =>
                CommonException::new(ErrorKind::NotFound, "not found"),
            _ => CommonException::new(ErrorKind::Unknown, "unknown error"),
        }
    }
}

impl From<serde_json::Error> for CommonException {
    fn from(err: serde_json::Error) -> Self {
        log::error!("invalid json: {}", err);
        Self::new(ErrorKind::InvalidJson, "invalid json")
    }
}

impl CommonException {
    pub fn new(kind: ErrorKind, msg: &str) -> CommonException {
        CommonException {
            kind,
            message: msg.to_string(),
        }
    }
}

impl Error for CommonException {
    fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    fn msg(&self) -> String {
        self.message.clone()
    }

    fn code(&self) -> u16 {
        500
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionException {
    pub kind: ErrorKind,
    pub msg: String,
}

impl Error for ExecutionException {
    fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    fn msg(&self) -> String {
        self.msg.clone()
    }

    fn code(&self) -> u16 {
        500
    }
}

impl ExecutionException {
    pub fn invalid_job_graph(job_id: &types::JobID) -> ExecutionException {
        ExecutionException {
            kind: ErrorKind::InvalidJobGraph,
            msg: format!("Invalid job graph with id {:?}", job_id),
        }
    }

    pub fn fail_send_event_to_job_graph(job_id: &types::JobID) -> ExecutionException {
        ExecutionException {
            kind: ErrorKind::SendGraphEventFailed,
            msg: format!("graph event sent failed to id {:?}", job_id),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskWorkerError {
    ChannelDisconnected,
    ChannelEmpty,
    ExecutionError(String),
}

impl From<mpsc::error::TryRecvError> for TaskWorkerError {
    fn from(err: mpsc::error::TryRecvError) -> Self {
        match err {
            mpsc::error::TryRecvError::Empty => TaskWorkerError::ChannelEmpty,
            mpsc::error::TryRecvError::Disconnected => TaskWorkerError::ChannelDisconnected
        }
    }
}

impl From<ExecutionException> for TaskWorkerError {
    fn from(err: ExecutionException) -> Self {
        Self::ExecutionError(err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PipelineError {
    pub kind: ErrorKind,
    pub msg: String,
}
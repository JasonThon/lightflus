use core::fmt;
use std::io;

use proto::{
    common::{ErrorCode, ResourceId, Response},
    common_impl::DataflowValidateError,
};

use rdkafka::error::KafkaError;
use tokio::sync::mpsc;

use crate::{event::LocalEvent, types::SinkId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
}

impl From<&mut tonic::transport::Error> for ApiError {
    fn from(err: &mut tonic::transport::Error) -> Self {
        Self {
            code: ErrorCode::InternalError as i32,
            msg: err.to_string(),
        }
    }
}

impl From<tonic::transport::Error> for ApiError {
    fn from(err: tonic::transport::Error) -> Self {
        Self {
            code: ErrorCode::InternalError as i32,
            msg: err.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(serde_json::to_string(self).unwrap().as_str())
    }
}

impl ApiError {
    pub fn from_error<T: Error>(err: T) -> Self {
        Self {
            code: err.code() as i32,
            msg: err.msg(),
        }
    }

    pub fn into_tonic_status(&self) -> tonic::Status {
        todo!()
    }
}

impl From<tonic::Status> for ApiError {
    fn from(err: tonic::Status) -> Self {
        let msg = format!("{}", err);
        match err.code() {
            tonic::Code::InvalidArgument => Self {
                code: ErrorCode::RpcInvalidArgument as i32,
                msg,
            },
            tonic::Code::NotFound => Self {
                code: ErrorCode::ResourceNotFound as i32,
                msg,
            },
            tonic::Code::PermissionDenied => Self {
                code: ErrorCode::RpcPermissionDenied as i32,
                msg,
            },
            tonic::Code::Unauthenticated => Self {
                code: ErrorCode::RpcUnauthorized as i32,
                msg,
            },
            _ => Self {
                code: ErrorCode::InternalError as i32,
                msg,
            },
        }
    }
}

impl Error for DataflowValidateError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::DataflowValidateFailed
    }

    fn msg(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn code(&self) -> ErrorCode {
        match self {
            DataflowValidateError::OperatorInfoMissing(_) => ErrorCode::DataflowOperatorInfoMissing,
            DataflowValidateError::CyclicDataflow => ErrorCode::CyclicDataflow,
            DataflowValidateError::OperatorDetailMissing(_) => {
                ErrorCode::DataflowOperatorInfoMissing
            }
            _ => ErrorCode::DataflowConfigurationMissing,
        }
    }
}

pub trait Error {
    fn to_string(&self) -> String {
        serde_json::to_string(&ApiError {
            code: self.code() as i32,
            msg: format!("Error Kind: {:?}. Message: {}", self.kind(), self.msg()),
        })
        .unwrap()
    }

    fn kind(&self) -> ErrorKind;

    fn msg(&self) -> String;

    fn code(&self) -> ErrorCode;
}

impl From<&Response> for ApiError {
    fn from(_resp: &Response) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct CommonException {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    NoAvailableWorker,
    InvalidDataflow,
    SinkLocalEventFailure,
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
    InvalidJson,
    SaveDataflowFailed,
    GetDataflowFailed,
    UnexpectedWireType,
    IncorrectWireTag,
    IncompleteWireMap,
    IncorrectVarint,
    Utf8Error,
    InvalidEnumValue,
    OverRecursionLimit,
    TruncatedMessage,
    MessageNotInitialized,
    OpenDBFailed,
    DeleteDataflowFailed,
    Other,
    DataflowValidateFailed,
}

impl From<std::io::Error> for CommonException {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::TimedOut => CommonException::new(ErrorKind::Timeout, "request timeout"),
            io::ErrorKind::AddrNotAvailable => {
                CommonException::new(ErrorKind::InvalidEndpoint, "endpoint address is invalid")
            }
            io::ErrorKind::ConnectionRefused => {
                CommonException::new(ErrorKind::ConnectionRefused, "connection refused")
            }
            io::ErrorKind::ConnectionAborted => {
                CommonException::new(ErrorKind::ConnectionAborted, "connection abort")
            }
            io::ErrorKind::UnexpectedEof => {
                CommonException::new(ErrorKind::UnexpectedEof, "unexpected eof")
            }
            io::ErrorKind::NotConnected => {
                CommonException::new(ErrorKind::NotConnected, "not connected")
            }
            io::ErrorKind::NotFound => CommonException::new(ErrorKind::NotFound, "not found"),
            _ => CommonException::new(ErrorKind::Unknown, "unknown error"),
        }
    }
}

impl From<serde_json::Error> for CommonException {
    fn from(err: serde_json::Error) -> Self {
        Self::new(
            ErrorKind::InvalidJson,
            format!("invalid json: {}", err).as_str(),
        )
    }
}

impl CommonException {
    pub fn new(kind: ErrorKind, msg: &str) -> CommonException {
        CommonException {
            kind,
            message: msg.to_string(),
        }
    }

    pub fn to_api_error(&self) -> Result<(), ApiError> {
        todo!()
    }
}

impl Error for CommonException {
    fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    fn msg(&self) -> String {
        self.message.clone()
    }

    fn code(&self) -> ErrorCode {
        ErrorCode::InternalError
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

    fn code(&self) -> ErrorCode {
        ErrorCode::InternalError
    }
}

impl ExecutionException {
    pub fn invalid_dataflow(job_id: &ResourceId) -> ExecutionException {
        ExecutionException {
            kind: ErrorKind::InvalidDataflow,
            msg: format!("Invalid job graph with id {:?}", job_id),
        }
    }

    pub fn sink_local_event_failure(
        job_id: &ResourceId,
        event: &LocalEvent,
        sink_id: SinkId,
        err_msg: String,
    ) -> ExecutionException {
        ExecutionException {
            kind: ErrorKind::SinkLocalEventFailure,
            msg: format!(
                "job id {job_id:?} sink msg {event:?} to {} failed. {err_msg:?}",
                sink_id
            ),
        }
    }
}

#[derive(Debug)]
pub enum TaskWorkerError {
    DataflowValidateError(DataflowValidateError),
    ChannelDisconnected,
    ChannelEmpty,
    ExecutionError(String),
    EventSendFailure(String),
}

impl From<mpsc::error::TryRecvError> for TaskWorkerError {
    fn from(err: mpsc::error::TryRecvError) -> Self {
        match err {
            mpsc::error::TryRecvError::Empty => TaskWorkerError::ChannelEmpty,
            mpsc::error::TryRecvError::Disconnected => TaskWorkerError::ChannelDisconnected,
        }
    }
}

impl From<ExecutionException> for TaskWorkerError {
    fn from(err: ExecutionException) -> Self {
        Self::ExecutionError(err.to_string())
    }
}

impl TaskWorkerError {
    pub fn into_grpc_status(&self) -> tonic::Status {
        todo!()
    }
}

#[derive(Debug)]
pub struct KafkaException {
    pub err: KafkaError,
}

impl std::fmt::Display for KafkaException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

#[derive(Debug)]
pub enum RedisException {
    ConnectFailed(String),
    SetValueFailed(String),
    SetMultipleValueFailed(String),
    GetValueFailed(String),
    DelValueFailed(String),
}

use core::fmt;
use std::io;

use proto::common::common::{ErrorCode, ResourceId, Response};

use rdkafka::error::KafkaError;
use tokio::sync::mpsc;

use crate::{event::LocalEvent, types::SinkId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub msg: String,
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
}

impl Error for DataflowValidateError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::DataflowValidateFailed
    }

    fn msg(&self) -> String {
        match self {
            DataflowValidateError::OperatorInfoMissing(msg) => msg.clone(),
        }
    }

    fn code(&self) -> ErrorCode {
        match self {
            DataflowValidateError::OperatorInfoMissing(_) => {
                ErrorCode::ERROR_CODE_DATAFLOW_OPERATOR_INFO_MISSING
            }
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

impl From<grpcio::Error> for ApiError {
    fn from(err: grpcio::Error) -> Self {
        match err {
            grpcio::Error::RpcFailure(status) => {
                if status.code() == grpcio::RpcStatusCode::DEADLINE_EXCEEDED {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RESOURCE_NOT_FOUND as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::NOT_FOUND {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RESOURCE_NOT_FOUND as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::UNIMPLEMENTED {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RPC_UNIMPLEMENTED as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::UNAVAILABLE {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RPC_UNAVAILABLE as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::UNAUTHENTICATED {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RPC_UNAUTHORIZED as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::INVALID_ARGUMENT {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RPC_INVALID_ARGUMENT as i32,
                        msg: status.message().to_string(),
                    };
                }

                if status.code() == grpcio::RpcStatusCode::PERMISSION_DENIED {
                    return ApiError {
                        code: ErrorCode::ERROR_CODE_RPC_PERMISSION_DENIED as i32,
                        msg: status.message().to_string(),
                    };
                }

                ApiError {
                    code: ErrorCode::ERROR_CODE_INTERNAL_ERROR as i32,
                    msg: status.message().to_string(),
                }
            }
            grpcio::Error::RpcFinished(_) => ApiError {
                code: ErrorCode::ERROR_CODE_TOO_MANY_REQUEST as i32,
                msg: format!("{}", err),
            },
            grpcio::Error::RemoteStopped => ApiError {
                code: ErrorCode::ERROR_CODE_RPC_UNAVAILABLE as i32,
                msg: format!("{}", err),
            },
            grpcio::Error::BindFail(_, _) => ApiError {
                code: ErrorCode::ERROR_CODE_RPC_BIND_FAILED as i32,
                msg: format!("{}", err),
            },
            grpcio::Error::GoogleAuthenticationFailed => ApiError {
                code: ErrorCode::ERROR_CODE_GOOGLE_AUTH_FAILED as i32,
                msg: format!("{}", err),
            },
            _ => ApiError {
                code: ErrorCode::ERROR_CODE_INTERNAL_ERROR as i32,
                msg: format!("{}", err),
            },
        }
    }
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

impl From<protobuf::ProtobufError> for CommonException {
    fn from(err: protobuf::ProtobufError) -> Self {
        match err {
            protobuf::ProtobufError::IoError(e) => Self::from(e),
            protobuf::ProtobufError::WireError(e) => Self::from(e),
            protobuf::ProtobufError::Utf8(e) => Self {
                kind: ErrorKind::Utf8Error,
                message: format!(
                    "utf8 error. {}: {}. {}: {:?}",
                    "valid UTF-8 start from",
                    e.valid_up_to(),
                    "error length",
                    e.error_len()
                ),
            },
            protobuf::ProtobufError::MessageNotInitialized { message } => Self {
                kind: ErrorKind::MessageNotInitialized,
                message: message.to_string(),
            },
        }
    }
}

impl From<protobuf::error::WireError> for CommonException {
    fn from(err: protobuf::error::WireError) -> Self {
        match err {
            protobuf::error::WireError::UnexpectedEof => Self {
                kind: ErrorKind::UnexpectedEof,
                message: "unexpected eof".to_string(),
            },
            protobuf::error::WireError::UnexpectedWireType(t) => Self {
                kind: ErrorKind::UnexpectedWireType,
                message: format!("UnexpectedWireType: {:?}", t),
            },
            protobuf::error::WireError::IncorrectTag(v) => Self {
                kind: ErrorKind::IncorrectWireTag,
                message: format!("IncorrectWireTag: {}", v),
            },
            protobuf::error::WireError::IncompleteMap => Self {
                kind: ErrorKind::IncompleteWireMap,
                message: "IncompleteWireMap".to_string(),
            },
            protobuf::error::WireError::IncorrectVarint => Self {
                kind: ErrorKind::IncorrectVarint,
                message: "IncorrectVarint".to_string(),
            },
            protobuf::error::WireError::Utf8Error => Self {
                kind: ErrorKind::Utf8Error,
                message: "uft8 error".to_string(),
            },
            protobuf::error::WireError::InvalidEnumValue(v) => Self {
                kind: ErrorKind::InvalidEnumValue,
                message: format!("InvalidEnumValue error. enum: {}", v),
            },
            protobuf::error::WireError::OverRecursionLimit => Self {
                kind: ErrorKind::OverRecursionLimit,
                message: "OverRecursionLimit error".to_string(),
            },
            protobuf::error::WireError::TruncatedMessage => Self {
                kind: ErrorKind::TruncatedMessage,
                message: "TruncatedMessage error".to_string(),
            },
            protobuf::error::WireError::Other => Self {
                kind: ErrorKind::Other,
                message: "Other error".to_string(),
            },
        }
    }
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
        ErrorCode::ERROR_CODE_INTERNAL_ERROR
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
        ErrorCode::ERROR_CODE_INTERNAL_ERROR
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
            mpsc::error::TryRecvError::Disconnected => TaskWorkerError::ChannelDisconnected,
        }
    }
}

impl From<ExecutionException> for TaskWorkerError {
    fn from(err: ExecutionException) -> Self {
        Self::ExecutionError(err.to_string())
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

pub enum DataflowValidateError {
    OperatorInfoMissing(String),
}

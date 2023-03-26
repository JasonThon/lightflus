use std::{fmt::Display, io};

use proto::{common::ErrorCode, common_impl::DataflowValidateError};

use rdkafka::error::KafkaError;
use tonic::metadata::MetadataValue;

pub type BizCode = i32;
pub type ErrorTypeCode = i32;

const X_LIGHTFLUS_CODE_METADATA_KEY: &str = "x-lightflus-code";
const LIGHTFLUS_RPC_CODE: &str = "999";

#[derive(Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BizError {
    pub biz_code: BizCode,
    pub error_code: ErrorTypeCode,
    pub message: String,
}

impl Display for BizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct RpcError {
    pub biz_err: BizError,
    pub status: tonic::Status,
}

impl Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl RpcError {
    pub fn into_tonic_status(&self) -> tonic::Status {
        match serde_json::to_string(&self.biz_err).map(|message| {
            let r = tonic::Status::new(self.status.code(), message);
            r.metadata_mut().append(
                X_LIGHTFLUS_CODE_METADATA_KEY,
                MetadataValue::from_static(LIGHTFLUS_RPC_CODE),
            );
            r
        }) {
            Ok(status) => status,
            Err(err) => tonic::Status::internal(err.to_string()),
        }
    }

    pub fn parse(status: tonic::Status) -> Result<Self, tonic::Status> {
        if matches!(status
            .metadata()
            .get(X_LIGHTFLUS_CODE_METADATA_KEY), Some(val) if val == LIGHTFLUS_RPC_CODE)
        {
            serde_json::from_str::<BizError>(status.message())
                .map(|biz_err| Self { biz_err, status })
                .map_err(|err| tonic::Status::internal(err.to_string()))
        } else {
            Err(status)
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
    fn kind(&self) -> ErrorKind;

    fn msg(&self) -> String;

    fn code(&self) -> ErrorCode;
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

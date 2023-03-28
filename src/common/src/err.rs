use std::fmt::Display;

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
        f.write_fmt(format_args!(
            "code [{}], errorCode [{}], message [{}]",
            self.biz_code, self.error_code, self.message
        ))
    }
}

#[derive(Clone, Debug)]
pub struct RpcError {
    pub biz_err: BizError,
    pub status: tonic::Status,
}

impl Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "RpcError status: [{}], details: [{}]",
            self.status, self.biz_err
        ))
    }
}

impl RpcError {
    pub fn into_tonic_status(&self) -> tonic::Status {
        match serde_json::to_string(&self.biz_err).map(|message| {
            let mut r = tonic::Status::new(self.status.code(), message);
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
    fn msg(&self) -> String;

    fn code(&self) -> ErrorCode;
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

use proto::common::Response;

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

impl From<&Response> for ApiError {
    fn from(_resp: &Response) -> Self {
        todo!()
    }
}

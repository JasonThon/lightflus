pub mod taskmanager {
    use common::err::{BizCode, BizError, RpcError};
    use proto::common_impl::DataflowValidateError;
    use tokio::sync::mpsc::error::TryRecvError;

    pub const TASK_MANAGER_BIZ_CODE: BizCode = 200;

    pub fn resource_id_unprovided() -> RpcError {
        RpcError {
            biz_err: BizError {
                biz_code: TASK_MANAGER_BIZ_CODE,
                error_code: 1,
                message: "no resource id provided".to_string(),
            },
            status: tonic::Status::invalid_argument("no resource id provided"),
        }
    }

    pub fn execution_id_unprovided() -> RpcError {
        RpcError {
            biz_err: BizError {
                biz_code: TASK_MANAGER_BIZ_CODE,
                error_code: 2,
                message: "no execution id provided".to_string(),
            },
            status: tonic::Status::invalid_argument("no execution id provided"),
        }
    }

    pub fn no_found_worker() -> RpcError {
        RpcError {
            biz_err: BizError {
                biz_code: TASK_MANAGER_BIZ_CODE,
                error_code: 3,
                message: "no valid worker found".to_string(),
            },
            status: tonic::Status::not_found("no valid worker found"),
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

    impl From<TryRecvError> for TaskWorkerError {
        fn from(err: TryRecvError) -> Self {
            match err {
                TryRecvError::Empty => TaskWorkerError::ChannelEmpty,
                TryRecvError::Disconnected => TaskWorkerError::ChannelDisconnected,
            }
        }
    }

    impl TaskWorkerError {
        pub fn into_grpc_status(&self) -> tonic::Status {
            let mut rpc_err = RpcError {
                biz_err: BizError {
                    biz_code: TASK_MANAGER_BIZ_CODE,
                    error_code: 0,
                    message: Default::default(),
                },
                status: tonic::Status::unknown(""),
            };
            match self {
                Self::DataflowValidateError(err) => {
                    rpc_err.status = tonic::Status::invalid_argument(format!(
                        "validate dataflow faield: {:?}",
                        err
                    ));
                    rpc_err.biz_err.error_code = 4;
                    rpc_err.biz_err.message = format!("validate dataflow faield: {:?}", err);
                }
                TaskWorkerError::ChannelDisconnected => {
                    rpc_err.status = tonic::Status::internal("channel disconnected");
                    rpc_err.biz_err.error_code = 5;
                    rpc_err.biz_err.message = "channel disconnected".to_string();
                }
                TaskWorkerError::ChannelEmpty => {
                    rpc_err.status = tonic::Status::internal("channel empty");
                    rpc_err.biz_err.error_code = 6;
                    rpc_err.biz_err.message = "channel empty".to_string();
                }
                TaskWorkerError::ExecutionError(err) => {
                    rpc_err.status = tonic::Status::aborted(format!("execution error: {:?}", err));
                    rpc_err.biz_err.error_code = 7;
                    rpc_err.biz_err.message = format!("execution error: {:?}", err);
                }
                TaskWorkerError::EventSendFailure(err) => {
                    rpc_err.status =
                        tonic::Status::internal(format!("event sent error: {:?}", err));
                    rpc_err.biz_err.error_code = 8;
                    rpc_err.biz_err.message = format!("event sent error: {:?}", err);
                }
            }
            rpc_err.into_tonic_status()
        }
    }
}

pub mod coordinator {
    use common::err::{BizCode, BizError, RpcError};
    use proto::common::{DataflowStatus, ResourceId};

    pub const COORDINATOR_BIZ_CODE: BizCode = 100;

    pub fn unexpected_dataflow_staus(status: &DataflowStatus) -> RpcError {
        let message = format!("unexpected dataflow status {:?}", status);
        RpcError {
            biz_err: BizError {
                biz_code: COORDINATOR_BIZ_CODE,
                error_code: 1,
                message: message.clone(),
            },
            status: tonic::Status::internal(message.as_str()),
        }
    }

    pub fn task_deployment_err(message: &str) -> RpcError {
        RpcError {
            biz_err: BizError {
                biz_code: COORDINATOR_BIZ_CODE,
                error_code: 2,
                message: message.to_string(),
            },
            status: tonic::Status::internal(message),
        }
    }

    pub fn not_found_dataflow(job_id: &ResourceId) -> RpcError {
        let message = format!("not found dataflow {:?}", job_id);
        RpcError {
            biz_err: BizError {
                biz_code: COORDINATOR_BIZ_CODE,
                error_code: 3,
                message: message.clone(),
            },
            status: tonic::Status::not_found(message),
        }
    }
}

pub mod apiserver {
    use std::fmt;

    use common::err::Error;
    use proto::common::{ErrorCode, Response};

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
}

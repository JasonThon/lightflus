#[cfg(feature = "coordinator")]
pub mod coordinator;
#[cfg(feature = "errors")]
pub mod errors;
#[cfg(feature = "taskmanager")]
pub mod taskmanager;
#[cfg(feature = "apiserver")]
pub mod apiserver;

pub(crate) type RpcResponse<T> = Result<tonic::Response<T>, tonic::Status>;
pub(crate) type RpcRequest<T> = tonic::Request<T>;

pub fn new_rpc_response<T>(val: T) -> tonic::Response<T> {
    tonic::Response::new(val)
}

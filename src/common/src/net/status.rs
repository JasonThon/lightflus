pub const SUCCESS: &str = "success";

pub const SUCCESS_CODE: u16 = 0;

pub fn to_rpc_code(status: &str) -> grpcio::RpcStatusCode {
    return grpcio::RpcStatusCode::OK;
}
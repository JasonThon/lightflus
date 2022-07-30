use crate::types::stream::StreamGraph;

pub const SUCCESS: i32 = 200;
pub const BAD_REQUEST: i32 = 400;
pub const INTERNAL_SERVER_ERROR: i32 = 500;

pub fn to_io_error(err: grpcio::Error) -> std::io::Error {
    match err {
        grpcio::Error::Codec(err) => std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("codec error: {}", err.as_ref()),
        ),
        grpcio::Error::CallFailure(_) => std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "internal call failed",
        ),
        grpcio::Error::RpcFailure(_) => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::RpcFinished(_) => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::RemoteStopped => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::ShutdownFailed => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::BindFail(_, _) => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::QueueShutdown => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::GoogleAuthenticationFailed => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
        grpcio::Error::InvalidMetadata(_) => std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "",
        ),
    }
}

#[derive(Clone, Debug)]
pub struct ClientConfig {
    // address
    pub address: HostAddr,
    // timeout
    pub timeout: u32,
    // retry count
    pub retry: u32,
}

#[derive(Clone, Debug)]
pub struct HostAddr {
    pub host: String,
    pub port: u16,
}
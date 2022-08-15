use std::net::UdpSocket;

use proto::common::common::HostAddr;

pub const SUCCESS: i32 = 200;
pub const BAD_REQUEST: i32 = 400;
pub const INTERNAL_SERVER_ERROR: i32 = 500;
pub mod status;
pub mod cluster;

#[derive(Clone, Debug)]
pub struct ClientConfig {
    // address
    pub address: PersistableHostAddr,
    // timeout
    pub timeout: u32,
    // retry count
    pub retry: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, Default, Hash)]
pub struct PersistableHostAddr {
    pub host: String,
    pub port: u16,
}

impl PersistableHostAddr {
    pub fn as_uri(&self) -> String {
        format!("{}:{}", &self.host, self.port)
    }
}

pub fn to_host_addr(hashable: &PersistableHostAddr) -> HostAddr {
    let mut addr = HostAddr::new();
    addr.set_host(hashable.host.clone());
    addr.set_port(hashable.port as u32);
    addr
}

pub fn hostname() -> Option<String> {
    use std::process::Command;
    if cfg!(unix) || cfg!(windows) {
        let output = match Command::new("hostname").output() {
            Ok(o) => o,
            Err(_) => return None,
        };
        let mut s = String::from_utf8(output.stdout).unwrap();
        s.pop(); // pop '\n'
        Some(s)
    } else {
        None
    }
}

pub fn local_ip() -> Option<String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

use std::net::UdpSocket;

use proto::common::common::HostAddr;

pub const SUCCESS: i32 = 200;
pub const BAD_REQUEST: i32 = 400;
pub const INTERNAL_SERVER_ERROR: i32 = 500;
pub mod cluster;
pub mod status;

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

    fn is_valid(&self) -> bool {
        !self.host.is_empty() && self.port > 0
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

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_local_ip() {
        use super::local_ip;
        let option = local_ip();
        assert!(option.is_some());
        println!("{}", option.unwrap())
    }

    #[test]
    pub fn test_to_host_addr() {
        let mut addr = super::PersistableHostAddr {
            host: "198.0.0.1".to_string(),
            port: 8970,
        };

        let host_addr = super::to_host_addr(&addr);
        assert_eq!(host_addr.get_host(), "198.0.0.1");
        assert_eq!(host_addr.get_port(), 8970);

        assert_eq!(addr.as_uri().as_str(), "198.0.0.1:8970");
        assert!(addr.is_valid());

        addr.host = "".to_string();
        assert!(!addr.is_valid());

        addr.host = "198.0.0.1".to_string();
        addr.port = 0;
        assert!(!addr.is_valid());
    }

    #[test]
    pub fn test_hostname() {
        let host = super::hostname();
        assert!(host.is_some());
    }
}

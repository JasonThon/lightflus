use std::{net::UdpSocket, sync::atomic::AtomicU64, time::Duration};

use futures_util::Future;
use proto::{
    common::{Ack, ExecutionId, HostAddr, NodeType},
    common_impl::{ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway},
    worker_gateway::SafeTaskManagerRpcGateway,
};

use crate::utils;

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
        format!("http://{}:{}", &self.host, self.port)
    }

    fn is_valid(&self) -> bool {
        !self.host.is_empty() && self.port > 0
    }

    pub fn local(port: usize) -> Self {
        Self {
            host: hostname().unwrap_or_default(),
            port: port as u16,
        }
    }
}

pub fn to_host_addr(hashable: &PersistableHostAddr) -> HostAddr {
    HostAddr {
        host: hashable.host.clone(),
        port: hashable.port as u32,
    }
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

/// Heartbeat configurations
#[derive(serde::Deserialize, Clone, Debug, Copy)]
pub struct HeartbeatConfig {
    /// period of heartbeat, in seconds
    pub period: u64,
    /// timeout of rpc connection, in seconds
    pub connection_timeout: u64,
}

pub struct HeartbeatBuilder {
    pub node_addrs: Vec<(HostAddr, u64)>,
    pub period: u64,
}
impl HeartbeatBuilder {
    pub fn build(&self) -> Heartbeat {
        Heartbeat {
            gateways: self
                .node_addrs
                .iter()
                .map(|(host_addr, connect_timeout)| {
                    SafeTaskManagerRpcGateway::with_connection_timeout(host_addr, *connect_timeout)
                })
                .collect(),
            interval: tokio::time::interval(Duration::from_secs(self.period)),
            execution_id: None,
            current_heartbeat_id: AtomicU64::default(),
        }
    }
}

pub struct Heartbeat {
    gateways: Vec<SafeTaskManagerRpcGateway>,
    interval: tokio::time::Interval,
    execution_id: Option<ExecutionId>,
    current_heartbeat_id: AtomicU64,
}
impl Heartbeat {
    pub fn update_execution_id(&mut self, execution_id: ExecutionId) {
        self.execution_id = Some(execution_id)
    }
}

impl Future for Heartbeat {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let self_ = self.get_mut();
        loop {
            match self_.interval.poll_tick(cx) {
                std::task::Poll::Ready(_) => {
                    let now = utils::times::now();
                    tracing::debug!("heartbeat sent at time {:?}", now);

                    self_.gateways.iter().for_each(|gateway| {
                        let _ = gateway.receive_heartbeat(proto::common::Heartbeat {
                            heartbeat_id: self_
                                .current_heartbeat_id
                                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
                            timestamp: Some(prost_types::Timestamp {
                                seconds: now.timestamp(),
                                nanos: now.timestamp_subsec_nanos() as i32,
                            }),
                            node_type: NodeType::JobManager as i32,
                            execution_id: self_.execution_id.clone(),
                        });
                    })
                }
                _ => {}
            }
        }
    }
}

/// Ack Configurations
#[derive(serde::Deserialize, Clone, Debug)]
pub struct AckResponderBuilder {
    // deplay duration, in seconds
    pub delay: u64,
    // buffer ack queue size
    pub buf_size: u32,
    // ack nodes
    #[serde(default)]
    pub nodes: Vec<PersistableHostAddr>,
}

impl AckResponderBuilder {
    pub fn build<F: Fn(&Vec<PersistableHostAddr>) -> Vec<T>, T: ReceiveAckRpcGateway>(
        &self,
        f: F,
    ) -> (AckResponder<T>, tokio::sync::mpsc::Sender<Ack>) {
        let (tx, rx) = tokio::sync::mpsc::channel(self.buf_size as usize);
        (
            AckResponder {
                delay_interval: tokio::time::interval(Duration::from_secs(self.delay)),
                recv: rx,
                gateway: f(&self.nodes),
            },
            tx,
        )
    }
}

pub struct AckResponder<T: ReceiveAckRpcGateway> {
    delay_interval: tokio::time::Interval,
    recv: tokio::sync::mpsc::Receiver<Ack>,
    gateway: Vec<T>,
}

impl<T: ReceiveAckRpcGateway> Future for AckResponder<T> {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let self_ = self.get_mut();
        loop {
            match self_.delay_interval.poll_tick(cx) {
                std::task::Poll::Ready(_) => match self_.recv.poll_recv(cx) {
                    std::task::Poll::Ready(ack) => ack.into_iter().for_each(|ack| {
                        self_.gateway.iter().for_each(|gateway| {
                            let _ = gateway.receive_ack(ack.clone());
                        })
                    }),
                    std::task::Poll::Pending => {}
                },
                std::task::Poll::Pending => {}
            }
        }
    }
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
        assert_eq!(host_addr.host.as_str(), "198.0.0.1");
        assert_eq!(host_addr.port, 8970);

        assert_eq!(addr.as_uri().as_str(), "http://198.0.0.1:8970");
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

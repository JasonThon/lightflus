use std::{
    net::UdpSocket,
    pin::Pin,
    sync::atomic::{self, AtomicU64},
    task::{self, Poll},
    time::Duration,
};

use futures_util::{ready, Future, FutureExt};
use proto::common::{Ack, Heartbeat, HostAddr, NodeType, SubDataflowId};
use tokio::sync::mpsc;

use crate::{futures::join_all, types::ExecutorId, utils};

use self::gateway::{ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway};

pub(crate) const DEFAULT_RPC_TIMEOUT: u64 = 3;
pub(crate) const DEFAULT_CONNECT_TIMEOUT: u64 = 3;
pub mod cluster;
#[cfg(not(tarpaulin_include))]
pub mod gateway;

pub fn local(port: usize) -> HostAddr {
    HostAddr {
        host: hostname().unwrap_or_default(),
        port: port as u32,
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

/// Heartbeat Builder
///
/// How to build heartbeat sender
/// HeartbeatBuilder::build is used to build a heartbeat sender. This method has three arguments:
/// - First Arg: the host address of remote node
/// - Second Arg: rpc connection timeout
/// - Third Arg: rpc request timeout
///
/// [HeartbeatSender] implements [Future] which can be ran by:
/// - Tokio spawning
/// - async/await
///
/// # Example of Tokio spawning
///
/// ```
/// use common::net::{HeartbeatBuilder, gateway::taskmanager::SafeTaskManagerRpcGateway};
/// use proto::common::HostAddr;
///
/// #[tokio::main]
/// async fn main() {
///     let builder = HeartbeatBuilder {
///         nodes: vec![HostAddr {
///             host: "localhost".to_string(),
///             port: 8080
///         }],
///         period: 3,
///         connect_timeout: 3,
///         rpc_timeout: 3,
///     };
///     
///     let heartbeat = builder.build(|addr, connect_timeout, rpc_timeout| SafeTaskManagerRpcGateway::with_timeout(addr, connect_timeout, rpc_timeout));
///     let handler = tokio::spawn(heartbeat);
///     handler.abort();
/// }
/// ```
///
/// # Example of async/await
///
/// ```
/// use common::net::{HeartbeatBuilder, gateway::taskmanager::SafeTaskManagerRpcGateway};
/// use proto::common::HostAddr;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() {
///     let builder = HeartbeatBuilder {
///         nodes: vec![HostAddr {
///             host: "localhost".to_string(),
///             port: 8080
///         }],
///         period: 3,
///         connect_timeout: 3,
///         rpc_timeout: 3,
///     };
///     
///     let heartbeat = builder.build(|addr, connect_timeout, rpc_timeout| SafeTaskManagerRpcGateway::with_timeout(addr, connect_timeout, rpc_timeout));
///     let _ = tokio::time::timeout(Duration::from_secs(1), heartbeat);
/// }
/// ```
#[derive(serde::Deserialize, Clone, Debug)]
pub struct HeartbeatBuilder {
    /// period of heartbeat, in seconds
    pub period: u64,
    /// timeout of heartbeat rpc connection, in seconds
    pub connect_timeout: u64,
    /// timeout of heartbeat rpc request, in seconds
    pub rpc_timeout: u64,
}

impl HeartbeatBuilder {
    pub fn build<F: Fn(&HostAddr, Duration, Duration) -> T, T: ReceiveHeartbeatRpcGateway>(
        &self,
        host_addr: &HostAddr,
        task_id: ExecutorId,
        f: F,
    ) -> HeartbeatSender<T> {
        HeartbeatSender {
            gateway: f(
                host_addr,
                Duration::from_secs(self.connect_timeout),
                Duration::from_secs(self.rpc_timeout),
            ),
            interval: tokio::time::interval(Duration::from_secs(self.period)),
            execution_id: None,
            current_heartbeat_id: AtomicU64::default(),
            task_id,
        }
    }
}

pub struct HeartbeatSender<T: ReceiveHeartbeatRpcGateway> {
    gateway: T,
    interval: tokio::time::Interval,
    execution_id: Option<SubDataflowId>,
    current_heartbeat_id: AtomicU64,
    task_id: ExecutorId,
}
impl<T: ReceiveHeartbeatRpcGateway> HeartbeatSender<T> {
    pub fn update_execution_id(&mut self, execution_id: SubDataflowId) {
        self.execution_id = Some(execution_id)
    }
}

impl<T: ReceiveHeartbeatRpcGateway> Future for HeartbeatSender<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        ready!(Pin::new(&mut this.interval).poll_tick(cx));
        let now = utils::times::now();
        tracing::debug!("heartbeat sent at time {:?}", now);
        let mut future = this.gateway.receive_heartbeat(Heartbeat {
            heartbeat_id: this
                .current_heartbeat_id
                .fetch_add(1, atomic::Ordering::SeqCst),
            timestamp: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            }),
            node_type: NodeType::JobManager as i32,
            subdataflow_id: this.execution_id.clone(),
            task_id: 0,
        });
        while let false = future.poll_unpin(cx).is_ready() {}
        Poll::Pending
    }
}

/// The builder of [AckResponder] which is also the configuration of ACK
///
/// AckResponderBuilder::build has three arguments:
/// - First Arg: the host address of remote node
/// - Second Arg: rpc connection timeout
/// - Third Arg: rpc request timeout
///
/// It will return two values:
/// - a new [AckResponder]
/// - a [mpsc::Sender] channel for [Ack] messages. Users can trigger ack by send an [Ack] message into it.
///
/// [AckResponder] implements [Future]. Users can run an [AckResponder] by:
/// - Tokio spawn
/// - async/await
///
/// # Example of Tokio spawn
/// ```
/// use common::net::{AckResponderBuilder, gateway::taskmanager::SafeTaskManagerRpcGateway};
/// use proto::common::HostAddr;
///
/// #[tokio::main]
/// async fn main() {
///     let builder = AckResponderBuilder {
///         delay: 3,
///         buf_size: 10,
///         nodes: vec![HostAddr {
///             host: "localhost".to_string(),
///             port: 8080
///         }],
///         connect_timeout: 3,
///         rpc_timeout: 3
///     };
///     
///     let (responder, _) = builder.build(|addr, connect_timeout, rpc_timeout| SafeTaskManagerRpcGateway::with_timeout(addr, connect_timeout, rpc_timeout));
///     let _ = tokio::spawn(responder);
/// }
/// ```
///
/// # Example of Tokio spwan
/// ```
/// use common::net::{AckResponderBuilder, gateway::taskmanager::SafeTaskManagerRpcGateway};
/// use std::time::Duration;
/// use proto::common::HostAddr;
///
/// #[tokio::main]
/// async fn main() {
///     let builder = AckResponderBuilder {
///         delay: 3,
///         buf_size: 10,
///         nodes: vec![HostAddr {
///             host: "localhost".to_string(),
///             port: 8080
///         }],
///         connect_timeout: 3,
///         rpc_timeout: 3
///     };
///     
///     let (responder, _) = builder.build(|addr, connect_timeout, rpc_timeout| SafeTaskManagerRpcGateway::with_timeout(addr, connect_timeout, rpc_timeout));
///     let _ = tokio::time::timeout(Duration::from_secs(1), responder);
/// }
/// ```
#[derive(serde::Deserialize, Clone, Debug)]
pub struct AckResponderBuilder {
    // deplay duration, in seconds
    pub delay: u64,
    // buffer ack queue size
    pub buf_size: usize,
    // ack nodes
    pub nodes: Vec<HostAddr>,
    /// timeout of ack rpc connection, in seconds
    pub connect_timeout: u64,
    /// timeout of ack rpc request, in seconds
    pub rpc_timeout: u64,
}

impl AckResponderBuilder {
    pub fn build<F: Fn(&HostAddr, Duration, Duration) -> T, T: ReceiveAckRpcGateway>(
        &self,
        host_addr: &HostAddr,
        f: F,
    ) -> (AckResponder<T>, mpsc::Sender<Ack>) {
        let (tx, rx) = mpsc::channel(self.buf_size);
        (
            AckResponder {
                delay_interval: tokio::time::interval(Duration::from_secs(self.delay)),
                recv: rx,
                gateway: f(
                    host_addr,
                    Duration::from_secs(self.connect_timeout),
                    Duration::from_secs(self.rpc_timeout),
                ),
            },
            tx,
        )
    }
}

pub struct AckResponder<T: ReceiveAckRpcGateway> {
    delay_interval: tokio::time::Interval,
    recv: mpsc::Receiver<Ack>,
    gateway: T,
}

impl<T: ReceiveAckRpcGateway> Future for AckResponder<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        ready!(Pin::new(&mut this.delay_interval).poll_tick(cx));
        let mut all_ack_futures = vec![];
        loop {
            match this.recv.poll_recv(cx) {
                Poll::Ready(Some(ack)) => {
                    let future = this.gateway.receive_ack(ack.clone());
                    all_ack_futures.push(future);
                }
                Poll::Ready(None) => continue,
                _ => {
                    join_all(cx, &mut all_ack_futures, |r| match r {
                        Ok(_) => todo!(),
                        Err(status) => {}
                    });
                    return Poll::Pending;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use chrono::{Duration, Timelike};
    use proto::common::{ack::AckType, Ack, HostAddr, NodeType, ResourceId, SubDataflowId};

    use crate::net::gateway::MockRpcGateway;

    use super::HeartbeatBuilder;

    #[test]
    pub fn test_local_ip() {
        use super::local_ip;
        let option = local_ip();
        assert!(option.is_some());
        println!("{}", option.unwrap())
    }

    #[test]
    pub fn test_hostname() {
        let host = super::hostname();
        assert!(host.is_some());
    }

    #[tokio::test]
    async fn test_ack_success() {
        use super::AckResponderBuilder;

        let builder = AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            nodes: vec![HostAddr {
                host: "198.0.0.1".to_string(),
                port: 8970,
            }],
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let (gateway, mut rx, _) = MockRpcGateway::new(builder.buf_size, 10);

        let (responder, tx) = builder.build(|_, _, _| gateway.clone());

        let handler = tokio::spawn(responder);
        // send first time
        {
            let result = tx
                .send(Ack {
                    timestamp: None,
                    ack_type: AckType::Heartbeat as i32,
                    node_type: NodeType::JobManager as i32,
                    execution_id: None,
                    request_id: None,
                })
                .await;
            let start = chrono::Utc::now();
            assert!(result.is_ok());

            let result = rx.recv().await;
            let end = chrono::Utc::now();
            assert_eq!(
                result,
                Some(Ack {
                    timestamp: None,
                    ack_type: AckType::Heartbeat as i32,
                    node_type: NodeType::JobManager as i32,
                    execution_id: None,
                    request_id: None,
                })
            );

            let duration = end - start;
            assert!(duration <= Duration::seconds(1));
        }

        // send second time
        {
            let result = tx
                .send(Ack {
                    timestamp: None,
                    ack_type: AckType::Heartbeat as i32,
                    node_type: NodeType::JobManager as i32,
                    execution_id: None,
                    request_id: None,
                })
                .await;
            assert!(result.is_ok());
            let start = chrono::Utc::now();

            let result = rx.recv().await;
            let end = chrono::Utc::now();
            assert_eq!(
                result,
                Some(Ack {
                    timestamp: None,
                    ack_type: AckType::Heartbeat as i32,
                    node_type: NodeType::JobManager as i32,
                    execution_id: None,
                    request_id: None,
                })
            );

            let duration = end.second() - start.second();
            assert!(duration == 3)
        }

        handler.abort();
    }

    #[tokio::test]
    async fn test_heartbeat_success() {
        let builder = HeartbeatBuilder {
            nodes: vec![HostAddr {
                host: "11".to_string(),
                port: 11,
            }],
            period: 3,
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let (gateway, _, mut rx) = MockRpcGateway::new(10, 10);

        let heartbeat = builder.build(|_, _, _| gateway.clone());
        let handler = tokio::spawn(heartbeat);

        {
            let start = chrono::Utc::now();
            let result = rx.recv().await;
            let end = chrono::Utc::now();
            assert!(result.is_some());
            assert!(end - start <= Duration::seconds(1));
        }

        {
            let start = chrono::Utc::now();
            let result = rx.recv().await;
            let end = chrono::Utc::now();
            assert!(result.is_some());

            assert!(end.second() - start.second() == 3);
        }

        handler.abort()
    }

    #[tokio::test]
    async fn test_heartbeat_update_execution_id() {
        let builder = HeartbeatBuilder {
            nodes: vec![HostAddr {
                host: "11".to_string(),
                port: 11,
            }],
            period: 3,
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let (gateway, _, _) = MockRpcGateway::new(10, 10);

        let mut heartbeat = builder.build(|_, _, _| gateway.clone());
        heartbeat.update_execution_id(SubDataflowId {
            job_id: Some(ResourceId {
                resource_id: "resource_id".to_string(),
                namespace_id: "namespace_id".to_string(),
            }),
            sub_id: 1,
        });
        assert_eq!(
            heartbeat.execution_id,
            Some(SubDataflowId {
                job_id: Some(ResourceId {
                    resource_id: "resource_id".to_string(),
                    namespace_id: "namespace_id".to_string(),
                }),
                sub_id: 1,
            })
        )
    }
}

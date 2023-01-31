use proto::common::{Ack, Heartbeat, HostAddr, Response};
use tokio::sync::mpsc;
use tonic::async_trait;

/// Rpc Gateway trait. All Rpc clients should implement this trait
pub trait RpcGateway: Unpin {
    fn get_host_addr(&self) -> &HostAddr;
}

/// Trait for [RpcGateway] that must implements receive_ack rpc call
#[async_trait]
pub trait ReceiveAckRpcGateway: RpcGateway {
    async fn receive_ack(&self, req: Ack) -> Result<Response, tonic::Status>;
}

/// Trait for [RpcGateway] that must implements receive_heartbeat rpc call
#[async_trait]
pub trait ReceiveHeartbeatRpcGateway: RpcGateway {
    async fn receive_heartbeat(&self, request: Heartbeat) -> Result<Response, tonic::Status>;
}

#[derive(Clone)]
pub struct MockRpcGateway {
    ack_channel: mpsc::Sender<Ack>,
    heartbeat_channel: mpsc::Sender<Heartbeat>,
}

unsafe impl Send for MockRpcGateway {}
unsafe impl Sync for MockRpcGateway {}

#[async_trait]
impl ReceiveAckRpcGateway for MockRpcGateway {
    async fn receive_ack(&self, req: Ack) -> Result<Response, tonic::Status> {
        self.ack_channel
            .send(req)
            .await
            .map(|_| Response::ok())
            .map_err(|err| tonic::Status::data_loss(err.to_string()))
    }
}

impl RpcGateway for MockRpcGateway {
    fn get_host_addr(&self) -> &HostAddr {
        todo!()
    }
}

#[async_trait]
impl ReceiveHeartbeatRpcGateway for MockRpcGateway {
    async fn receive_heartbeat(&self, request: Heartbeat) -> Result<Response, tonic::Status> {
        self.heartbeat_channel
            .send(request)
            .await
            .map(|_| Response::ok())
            .map_err(|err| tonic::Status::data_loss(err.to_string()))
    }
}

impl MockRpcGateway {
    pub fn new(
        ack_buf_size: usize,
        heartbeat_buf_size: usize,
    ) -> (Self, mpsc::Receiver<Ack>, mpsc::Receiver<Heartbeat>) {
        let (ack_tx, ack_rx) = mpsc::channel(ack_buf_size);
        let (heartbeat_tx, heartbeat_rx) = mpsc::channel(heartbeat_buf_size);
        (
            Self {
                ack_channel: ack_tx,
                heartbeat_channel: heartbeat_tx,
            },
            ack_rx,
            heartbeat_rx,
        )
    }
}

pub mod taskmanager {
    use std::{sync::Arc, time::Duration};

    use prost::Message;
    use proto::{
        common::{Ack, Heartbeat, HostAddr, KeyedDataEvent, ResourceId, Response},
        taskmanager::{
            task_manager_api_client::TaskManagerApiClient, CreateSubDataflowRequest,
            CreateSubDataflowResponse, SendEventToOperatorResponse, StopDataflowResponse,
        },
    };
    use tonic::async_trait;

    use crate::net::DEFAULT_RPC_TIMEOUT;

    use super::{
        super::DEFAULT_CONNECT_TIMEOUT, ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway,
        RpcGateway,
    };

    /// A thread-safe RpcGateway wrapper for [`TaskManagerApiClient`]. It's also reponsible for concurrency control of client-side gRPC.
    /// [`SafeTaskWorkerRpcGateway`] ensures only one thread can call [`TaskManagerApiClient`] at the same time. Requests have to be sent FIFO, without any fault tolerance.
    /// [`SafeTaskWorkerRpcGateway`] can be shared in different threads safely.
    #[derive(Debug, Clone)]
    pub struct SafeTaskManagerRpcGateway {
        inner: Arc<tokio::sync::Mutex<Option<TaskManagerApiClient<tonic::transport::Channel>>>>,
        host_addr: HostAddr,
        connect_timeout: u64,
        rpc_timeout: u64,
    }

    unsafe impl Send for SafeTaskManagerRpcGateway {}
    unsafe impl Sync for SafeTaskManagerRpcGateway {}

    impl RpcGateway for SafeTaskManagerRpcGateway {
        fn get_host_addr(&self) -> &HostAddr {
            &self.host_addr
        }
    }

    impl Unpin for SafeTaskManagerRpcGateway {}

    #[async_trait]
    impl ReceiveAckRpcGateway for SafeTaskManagerRpcGateway {
        async fn receive_ack(&self, request: Ack) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                TaskManagerApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
                )
            });

            inner
                .receive_ack(tonic::Request::new(request))
                .await
                .map(|resp| resp.into_inner())
        }
    }

    #[async_trait]
    impl ReceiveHeartbeatRpcGateway for SafeTaskManagerRpcGateway {
        async fn receive_heartbeat(&self, request: Heartbeat) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                TaskManagerApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
                )
            });

            inner
                .receive_heartbeat(tonic::Request::new(request))
                .await
                .map(|resp| resp.into_inner())
        }
    }

    impl SafeTaskManagerRpcGateway {
        pub fn new(host_addr: &HostAddr) -> Self {
            let client = TaskManagerApiClient::with_connection_timeout(
                host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            );
            Self {
                inner: Arc::new(tokio::sync::Mutex::new(Some(client))),
                host_addr: host_addr.clone(),
                connect_timeout: DEFAULT_CONNECT_TIMEOUT,
                rpc_timeout: DEFAULT_RPC_TIMEOUT,
            }
        }

        pub fn with_timeout(host_addr: &HostAddr, connect_timeout: u64, rpc_timeout: u64) -> Self {
            let client = TaskManagerApiClient::with_connection_timeout(
                host_addr.as_uri(),
                Duration::from_secs(connect_timeout),
            );
            Self {
                inner: Arc::new(tokio::sync::Mutex::new(Some(client))),
                host_addr: host_addr.clone(),
                connect_timeout,
                rpc_timeout,
            }
        }

        pub async fn send_event_to_operator(
            &self,
            event: KeyedDataEvent,
        ) -> Result<SendEventToOperatorResponse, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                TaskManagerApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            let mut request = tonic::Request::new(event);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .send_event_to_operator(request)
                .await
                .map(|resp| resp.into_inner())
        }

        pub async fn stop_dataflow(
            &self,
            job_id: ResourceId,
        ) -> Result<StopDataflowResponse, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                TaskManagerApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            let mut request = tonic::Request::new(job_id);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .stop_dataflow(request)
                .await
                .map(|resp| resp.into_inner())
        }

        pub async fn create_sub_dataflow(
            &self,
            req: CreateSubDataflowRequest,
        ) -> Result<CreateSubDataflowResponse, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                TaskManagerApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            let mut request = tonic::Request::new(req);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .create_sub_dataflow(request)
                .await
                .map(|resp| resp.into_inner())
        }

        pub fn close(&mut self) {
            self.host_addr.clear();
            drop(self.inner.as_ref())
        }
    }
}

pub mod coordinator {
    use std::{sync::Arc, time::Duration};

    use tokio::sync::Mutex;
    use tonic::async_trait;

    use proto::{
        common::{Ack, Dataflow, Heartbeat, HostAddr, ResourceId, Response, TaskInfo},
        coordinator::{
            coordinator_api_client::CoordinatorApiClient, GetDataflowRequest, GetDataflowResponse,
        },
    };

    use crate::net::{DEFAULT_CONNECT_TIMEOUT, DEFAULT_RPC_TIMEOUT};

    use super::{ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway, RpcGateway};

    /// A thread-safe RpcGateway wrapper for [`CoordinatorApiClient`]. It's also reponsible for concurrency control of client-side gRPC.
    /// [`SafeCoordinatorRpcGateway`] ensures only one thread can call [`CoordinatorApiClient`] at the same time. Requests have to be sent FIFO, without any fault tolerance.
    /// [`SafeCoordinatorRpcGateway`] can be shared in different threads safely.
    #[derive(Debug, Clone)]
    pub struct SafeCoordinatorRpcGateway {
        inner: Arc<Mutex<Option<CoordinatorApiClient<tonic::transport::Channel>>>>,
        host_addr: HostAddr,
        rpc_timeout: u64,
        connect_timeout: u64,
    }

    impl RpcGateway for SafeCoordinatorRpcGateway {
        fn get_host_addr(&self) -> &HostAddr {
            &self.host_addr
        }
    }
    impl Unpin for SafeCoordinatorRpcGateway {}

    #[async_trait]
    impl ReceiveHeartbeatRpcGateway for SafeCoordinatorRpcGateway {
        async fn receive_heartbeat(&self, request: Heartbeat) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
                )
            });

            let mut request = tonic::Request::new(request);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .receive_heartbeat(request)
                .await
                .map(|resp| resp.into_inner())
        }
    }

    #[async_trait]
    impl ReceiveAckRpcGateway for SafeCoordinatorRpcGateway {
        async fn receive_ack(&self, req: Ack) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
                )
            });
            let mut request = tonic::Request::new(req);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .receive_ack(request)
                .await
                .map(|resp| resp.into_inner())
        }
    }

    impl SafeCoordinatorRpcGateway {
        pub fn new(host_addr: &HostAddr) -> Self {
            let client = futures_executor::block_on(CoordinatorApiClient::connect_with_timeout(
                host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            ));
            Self {
                inner: Arc::new(tokio::sync::Mutex::new(client.ok())),
                host_addr: host_addr.clone(),
                rpc_timeout: DEFAULT_RPC_TIMEOUT,
                connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            }
        }

        pub async fn create_dataflow(&self, dataflow: Dataflow) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            inner
                .create_dataflow(tonic::Request::new(dataflow))
                .await
                .map(|resp| resp.into_inner())
        }

        pub async fn terminate_dataflow(&self, req: ResourceId) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });
            let mut request = tonic::Request::new(req);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .terminate_dataflow(request)
                .await
                .map(|resp| resp.into_inner())
        }

        pub async fn get_dataflow(
            &self,
            req: GetDataflowRequest,
        ) -> Result<GetDataflowResponse, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            let mut request = tonic::Request::new(req);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .get_dataflow(request)
                .await
                .map(|resp| resp.into_inner())
        }

        pub async fn report_task_info(
            &mut self,
            request: TaskInfo,
        ) -> Result<Response, tonic::Status> {
            let mut guard = self.inner.lock().await;
            let inner = guard.get_or_insert_with(|| {
                CoordinatorApiClient::with_connection_timeout(
                    self.host_addr.as_uri(),
                    Duration::from_secs(self.connect_timeout),
                )
            });

            let mut request = tonic::Request::new(request);
            request.set_timeout(Duration::from_secs(self.rpc_timeout));

            inner
                .report_task_info(request)
                .await
                .map(|resp| resp.into_inner())
        }
    }
}

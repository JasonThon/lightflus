use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;
use tonic::async_trait;

use crate::{
    common::{Ack, Dataflow, Heartbeat, HostAddr, ResourceId, Response},
    common_impl::{ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway, RpcGateway},
    coordinator::{
        coordinator_api_client::CoordinatorApiClient, CreateDataflowResponse, GetDataflowRequest,
        GetDataflowResponse, TaskInfo, TerminateDataflowResponse,
    },
    DEFAULT_CONNECT_TIMEOUT,
};

/// A thread-safe RpcGateway wrapper for [`CoordinatorApiClient`]. It's also reponsible for concurrency control of client-side gRPC.
/// [`SafeCoordinatorRpcGateway`] ensures only one thread can call [`CoordinatorApiClient`] at the same time. Requests have to be sent FIFO, without any fault tolerance.
/// [`SafeCoordinatorRpcGateway`] can be shared in different threads safely.
#[derive(Debug, Clone)]
pub struct SafeCoordinatorRpcGateway {
    inner: Arc<Mutex<Option<CoordinatorApiClient<tonic::transport::Channel>>>>,
    host_addr: HostAddr,
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

        inner
            .receive_heartbeat(tonic::Request::new(request))
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

        inner
            .receive_ack(tonic::Request::new(req))
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
        }
    }

    pub async fn create_dataflow(
        &self,
        dataflow: Dataflow,
    ) -> Result<CreateDataflowResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            CoordinatorApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .create_dataflow(tonic::Request::new(dataflow))
            .await
            .map(|resp| resp.into_inner())
    }

    pub async fn terminate_dataflow(
        &self,
        req: ResourceId,
    ) -> Result<TerminateDataflowResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            CoordinatorApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .terminate_dataflow(tonic::Request::new(req))
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
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .get_dataflow(tonic::Request::new(req))
            .await
            .map(|resp| resp.into_inner())
    }

    pub async fn report_task_info(&mut self, request: TaskInfo) -> Result<Response, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            CoordinatorApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .report_task_info(tonic::Request::new(request))
            .await
            .map(|resp| resp.into_inner())
    }
}

use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::{
    common::{Dataflow, HostAddr, ProbeRequest, ProbeResponse},
    coordinator::{
        coordinator_api_client::CoordinatorApiClient, CreateDataflowResponse, GetDataflowRequest,
        GetDataflowResponse, TerminateDataflowRequest, TerminateDataflowResponse,
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
    pub async fn probe(&self, req: ProbeRequest) -> Result<ProbeResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            CoordinatorApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .probe(tonic::Request::new(req))
            .await
            .map(|resp| resp.into_inner())
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
        req: TerminateDataflowRequest,
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
}

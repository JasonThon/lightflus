use std::{sync::Arc, time::Duration};

use prost::Message;

use crate::{
    common::{HostAddr, KeyedDataEvent, ProbeRequest, ProbeResponse, ResourceId},
    worker::{
        task_worker_api_client::TaskWorkerApiClient, CreateSubDataflowRequest,
        CreateSubDataflowResponse, SendEventToOperatorResponse, StopDataflowResponse,
    },
    DEFAULT_CONNECT_TIMEOUT,
};

/// A thread-safe RpcGateway wrapper for [`TaskWorkerApiClient`]. It's also reponsible for concurrency control of client-side gRPC.
/// [`SafeTaskWorkerRpcGateway`] ensures only one thread can call [`TaskWorkerApiClient`] at the same time. Requests have to be sent FIFO, without any fault tolerance.
/// [`SafeTaskWorkerRpcGateway`] can be shared in different threads safely.
#[derive(Debug, Clone)]
pub struct SafeTaskManagerRpcGateway {
    inner: Arc<tokio::sync::Mutex<Option<TaskWorkerApiClient<tonic::transport::Channel>>>>,
    host_addr: HostAddr,
}

unsafe impl Send for SafeTaskManagerRpcGateway {}
unsafe impl Sync for SafeTaskManagerRpcGateway {}

impl SafeTaskManagerRpcGateway {
    pub fn new(host_addr: &HostAddr) -> Self {
        let client = TaskWorkerApiClient::with_connection_timeout(
            host_addr.as_uri(),
            Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
        );
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(client))),
            host_addr: host_addr.clone(),
        }
    }

    pub async fn probe(&self, req: ProbeRequest) -> Result<ProbeResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            TaskWorkerApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .probe(tonic::Request::new(req))
            .await
            .map(|resp| resp.into_inner())
    }

    pub async fn send_event_to_operator(
        &self,
        event: KeyedDataEvent,
    ) -> Result<SendEventToOperatorResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            TaskWorkerApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .send_event_to_operator(tonic::Request::new(event))
            .await
            .map(|resp| resp.into_inner())
    }

    pub async fn stop_dataflow(
        &self,
        job_id: ResourceId,
    ) -> Result<StopDataflowResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            TaskWorkerApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .stop_dataflow(tonic::Request::new(job_id))
            .await
            .map(|resp| resp.into_inner())
    }

    pub async fn create_sub_dataflow(
        &self,
        req: CreateSubDataflowRequest,
    ) -> Result<CreateSubDataflowResponse, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            TaskWorkerApiClient::with_connection_timeout(
                self.host_addr.as_uri(),
                Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
            )
        });

        inner
            .create_sub_dataflow(tonic::Request::new(req))
            .await
            .map(|resp| resp.into_inner())
    }

    pub fn close(&mut self) {
        self.host_addr.clear();
        drop(self.inner.as_ref())
    }
}

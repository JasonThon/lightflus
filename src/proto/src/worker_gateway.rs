use std::{sync::Arc, time::Duration};

use prost::Message;
use tonic::async_trait;

use crate::{
    common::{Ack, Heartbeat, HostAddr, KeyedDataEvent, ResourceId, Response},
    common_impl::{ReceiveAckRpcGateway, ReceiveHeartbeatRpcGateway, RpcGateway},
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

impl RpcGateway for SafeTaskManagerRpcGateway {
    fn get_host_addr(&self) -> &HostAddr {
        &self.host_addr
    }
}

#[async_trait]
impl ReceiveAckRpcGateway for SafeTaskManagerRpcGateway {
    async fn receive_ack(&self, request: Ack) -> Result<Response, tonic::Status> {
        let mut guard = self.inner.lock().await;
        let inner = guard.get_or_insert_with(|| {
            TaskWorkerApiClient::with_connection_timeout(
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
            TaskWorkerApiClient::with_connection_timeout(
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
        let client = TaskWorkerApiClient::with_connection_timeout(
            host_addr.as_uri(),
            Duration::from_secs(DEFAULT_CONNECT_TIMEOUT),
        );
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(client))),
            host_addr: host_addr.clone(),
        }
    }

    pub fn with_connection_timeout(host_addr: &HostAddr, connect_timeout: u64) -> Self {
        let client = TaskWorkerApiClient::with_connection_timeout(
            host_addr.as_uri(),
            Duration::from_secs(connect_timeout),
        );
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(Some(client))),
            host_addr: host_addr.clone(),
        }
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

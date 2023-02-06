use async_trait::async_trait;
use common::{
    event::{LocalEvent, StreamEvent},
    net::gateway::taskmanager::UnsafeTaskManagerRpcGateway,
};

#[async_trait]
pub trait OutEdge: Send {
    type Output;

    async fn send(&self, val: &Self::Output) -> Result<(), OutEdgeError>;
}

pub struct LocalOutEdge<T> {
    tx: tokio::sync::mpsc::Sender<T>,
}

unsafe impl<T> Send for LocalOutEdge<T> {}

impl<T> LocalOutEdge<T> {
    pub fn new(tx: tokio::sync::mpsc::Sender<T>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl<T: StreamEvent> OutEdge for LocalOutEdge<T> {
    type Output = T;

    async fn send(&self, val: &T) -> Result<(), OutEdgeError> {
        self.tx
            .send(val)
            .await
            .map_err(|err| OutEdgeError::SendToLocalFailed(err.to_string()))
    }
}

pub enum OutEdgeError {
    SendToLocalFailed(String),
}

impl ToString for OutEdgeError {
    fn to_string(&self) -> String {
        todo!()
    }
}

pub struct RemoteOutEdge {
    gateway: UnsafeTaskManagerRpcGateway,
}

impl RemoteOutEdge {
    pub fn new(gateway: UnsafeTaskManagerRpcGateway) -> Self {
        Self { gateway }
    }
}

unsafe impl Send for RemoteOutEdge {}

#[async_trait]
impl OutEdge for RemoteOutEdge {
    type Output = LocalEvent;

    async fn send(&self, val: &LocalEvent) -> Result<(), OutEdgeError> {
        todo!()
    }
}

/// The trait that represents an in-edge
#[async_trait]
pub trait InEdge: Send + Unpin {
    type Output;

    async fn receive_data_stream(&mut self) -> Option<Self::Output>;

    fn poll_recv_data_stream(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Output>>;
}

pub struct LocalInEdge<T> {
    rx: tokio::sync::mpsc::Receiver<T>,
}

unsafe impl<T> Send for LocalInEdge<T> {}

#[async_trait]
impl<T: Send> InEdge for LocalInEdge<T> {
    type Output = T;

    async fn receive_data_stream(&mut self) -> Option<T> {
        self.rx.recv().await
    }

    fn poll_recv_data_stream(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Output>> {
        self.rx.poll_recv(cx)
    }
}

impl<T> LocalInEdge<T> {
    pub fn new(rx: tokio::sync::mpsc::Receiver<T>) -> Self {
        Self { rx }
    }
}

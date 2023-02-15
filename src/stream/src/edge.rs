use std::marker::PhantomData;

use common::{
    event::{LocalEvent, StreamEvent},
    net::gateway::taskmanager::UnsafeTaskManagerRpcGateway,
};
use tonic::async_trait;

#[async_trait]
pub trait OutEdge: Send + Sync {
    type Output;

    async fn send(&self, val: Self::Output) -> Result<(), OutEdgeError>;
}

pub struct LocalOutEdge<T> {
    tx: tokio::sync::mpsc::Sender<bytes::Bytes>,
    _data_type: PhantomData<T>,
}

unsafe impl<T> Send for LocalOutEdge<T> {}
unsafe impl<T> Sync for LocalOutEdge<T> {}

impl<T> LocalOutEdge<T> {
    pub fn new(tx: tokio::sync::mpsc::Sender<bytes::Bytes>) -> Self {
        Self {
            tx,
            _data_type: PhantomData,
        }
    }
}

#[async_trait]
impl<T: StreamEvent> OutEdge for LocalOutEdge<T> {
    type Output = T;

    async fn send(&self, val: T) -> Result<(), OutEdgeError> {
        let mut buf = vec![];
        let mut serializer = rmp_serde::Serializer::new(&mut buf);
        val.serialize(&mut serializer)
            .map_err(|err| OutEdgeError::from(err))?;

        self.tx
            .send(bytes::Bytes::from(buf))
            .await
            .map_err(|err| OutEdgeError::SendToLocalFailed(err.to_string()))
    }
}

#[derive(Debug)]
pub enum OutEdgeError {
    SendToLocalFailed(String),
}

impl From<rmp_serde::encode::Error> for OutEdgeError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        todo!()
    }
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
unsafe impl Sync for RemoteOutEdge {}

#[async_trait]
impl OutEdge for RemoteOutEdge {
    type Output = LocalEvent;

    async fn send(&self, val: LocalEvent) -> Result<(), OutEdgeError> {
        todo!()
    }
}

/// The trait that represents an in-edge
#[async_trait]
pub trait InEdge: Send + Sync + Unpin {
    type Output;

    async fn receive_data_stream(&mut self) -> Option<Self::Output>;

    fn poll_recv_data_stream(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Output>>;
}

pub struct LocalInEdge<T> {
    rx: tokio::sync::mpsc::Receiver<bytes::Bytes>,
    _data_type: PhantomData<T>,
}

unsafe impl<T> Send for LocalInEdge<T> {}
unsafe impl<T> Sync for LocalInEdge<T> {}
impl<T> Unpin for LocalInEdge<T> {}

#[async_trait]
impl<T: StreamEvent> InEdge for LocalInEdge<T> {
    type Output = T;

    async fn receive_data_stream(&mut self) -> Option<T> {
        self.rx.recv().await.and_then(|buf| {
            T::from_slice(&buf)
                .map_err(|err| tracing::error!("{:?}", err))
                .ok()
        })
    }

    fn poll_recv_data_stream(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Output>> {
        match self.rx.poll_recv(cx) {
            std::task::Poll::Ready(buf) => std::task::Poll::Ready(buf.and_then(|buf| {
                T::from_slice(&buf)
                    .map_err(|err| tracing::error!("{:?}", err))
                    .ok()
            })),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl<T> LocalInEdge<T> {
    pub fn new(rx: tokio::sync::mpsc::Receiver<bytes::Bytes>) -> Self {
        Self {
            rx,
            _data_type: PhantomData,
        }
    }
}

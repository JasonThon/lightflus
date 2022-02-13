use std::io::Error;

use serde::Serialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

use crate::event;
use crate::http;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RpcDesc {
    pub port: u32,
    pub endpoint: EndpointKind,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum EndpointKind {
    EventDriven
}

pub struct RpcError {
    pub kind: ErrorKind,
    pub msg: String,
}

impl RpcError {
    pub(crate) fn invalid_codec() -> RpcError {
        RpcError {
            kind: ErrorKind::InvalidCodec,
            msg: "invalid codec".to_string(),
        }
    }
}

impl From<mpsc::error::TryRecvError> for RpcError {
    fn from(err: TryRecvError) -> Self {
        todo!()
    }
}

impl From<tokio::io::Error> for RpcError {
    fn from(_: Error) -> Self {
        todo!()
    }
}

pub enum ErrorKind {
    InvalidCodec
}

type Result<T> = std::result::Result<T, RpcError>;

pub trait RpcEndpoint<T, F> {
    fn try_send(&mut self, data: T) -> Result<F>;
    fn try_recv(&self) -> Result<T>;
}

pub trait RpcHandler<T> {
    fn handle(&self, cmd: T) -> Result<()>;
}

pub struct EventDrivenClient {
    rx: mpsc::UnboundedReceiver<event::Event>,
    conn: std::cell::RefCell<tokio::net::TcpStream>,
}

impl RpcEndpoint<event::Event, http::Response> for EventDrivenClient {
    fn try_send(&mut self, data: event::Event) -> Result<http::Response> {
        data.serialize(event::EventCodec::new())
            .map_err(|err| RpcError::invalid_codec())
            .and_then(|serial| self.conn.get_mut()
                .try_write(serial.as_bytes())
                .map_err(|err| RpcError::from(err))
                .map(|size| http::Response::ok())
            )
    }

    fn try_recv(&mut self) -> Result<event::Event> {
        self.rx.try_recv()
            .map_err(|err| RpcError::from(err))
    }
}

impl EventDrivenClient {
    pub fn new(rx: mpsc::UnboundedReceiver<event::Event>) -> EventDrivenClient {
        EventDrivenClient {
            rx,
            conn: Default::default(),
        }
    }

    pub async fn connect(&self, server_addr: &str) -> tokio::io::Result<tokio::net::TcpStream> {
        tokio::net::TcpStream::connect(server_addr).await
            .map(|stream| self.conn.replace(stream))
    }

    pub async fn start(&mut self) {
        loop {
            match self.try_recv() {
                Ok(cmd) => match self.try_send(cmd) {
                    Ok(_) => (),
                    Err(err) => {}
                },
                Err(err) => {}
            }
        }
    }
}
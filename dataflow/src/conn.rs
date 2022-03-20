use tokio::sync::mpsc::UnboundedSender;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::any::Any;
use crate::event;
use crate::event::ConnectorEvent;
use crate::types::SourceDesc;

pub struct Connector {
    table_id: String,
    header_id: String,
    tx: UnboundedSender<event::ConnectorEvent>,
    connector_type: ConnectorType,
}

impl Connector {
    pub fn new(table_id: String, header_id: String,
               connector_type: ConnectorType, tx: &UnboundedSender<ConnectorEvent>) -> Connector {
        Connector {
            table_id,
            header_id,
            tx: tx.clone(),
            connector_type,
        }
    }

    pub fn take_event(&self) -> Result<ConnectorEvent, super::err::ConnectionError> {
        todo!()
    }
}

impl std::future::Future for Connector {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let option = self.take_event();

            if option.is_err() && self.connector_type.type_id() == ConnectorType::Tableflow.type_id() {
                break;
            }

            let event = option.unwrap();
            match self.tx.send(event) {
                Ok(_) => log::info!(""),
                Err(err) => {}
            }
        }

        Poll::Ready(())
    }
}

pub enum ConnectorType {
    Tableflow {
        cli: data_client::tableflow_grpc::DataEngineClient,
        next_page: u32,
        limit: u32,
    },

    Redis {
        cli: redis::Client
    },
}

pub fn to_connector(source: &SourceDesc, table_id: &String,
                    header_id: &String,
                    tx: &UnboundedSender<event::ConnectorEvent>) -> Connector {
    match source {
        SourceDesc::Tableflow {
            host, port, page, limit
        } => Connector::new(
            table_id.clone(),
            header_id.clone(),
            ConnectorType::Tableflow {
                cli: data_client::new_data_engine_client(
                    data_client::DataEngineConfig {
                        host: host.to_string(),
                        port: port.clone(),
                    }
                ),
                next_page: page.clone(),
                limit: limit.clone(),
            },
            tx,
        ),

        SourceDesc::Redis {
            host, port, username, password, db
        } => Connector::new(
            table_id.clone(),
            header_id.clone(),
            ConnectorType::Redis {
                cli: redis::Client::open(format!("redis://{}:{}@{}:{}/{}", username, password, host, port, db))?
            },
            tx,
        )
    }
}

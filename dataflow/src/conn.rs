use std::time;
use tokio::sync::mpsc;
use crate::{err, event, types};
use crate::event::Event;

const DEFAULT_EVENT_TIME_DURATION: time::Duration = time::Duration::from_millis(1);
const DATAFLOW_EVENT_PATTERN: &str = "dataflow*";

pub enum ConnectorType {
    Tableflow {
        limit: u32,
        uri: String,
    },
    Redis {
        conn: redis::Connection,
    },
}

pub struct Connector {
    pub binders: Vec<types::Binder>,
    pub connector_type: ConnectorType,
    event_rx: mpsc::UnboundedReceiver<Vec<event::BinderEvent>>,
    disconnect_rx: mpsc::Receiver<event::Disconnect>,
    event_time_duration: Option<time::Duration>,
}

impl Connector {
    pub fn new(desc: &types::SourceDesc,
               event_rx: mpsc::UnboundedReceiver<Vec<event::BinderEvent>>,
               disconnect_rx: mpsc::Receiver<event::Disconnect>) -> Connector {
        match desc {
            types::SourceDesc::Tableflow { host, port, limit, event_time } =>
                Connector {
                    binders: vec![],
                    connector_type: ConnectorType::Tableflow {
                        limit: limit.clone(),
                        uri: format!("{}:{}", host, port),
                    },
                    event_rx,
                    disconnect_rx,
                    event_time_duration: event_time.map(|secs| time::Duration::from_secs(secs)),
                },
            types::SourceDesc::Redis {
                host, port,
                username, password,
                db
            } => Connector {
                binders: vec![],
                connector_type: ConnectorType::Redis {
                    conn: redis::Client::open(
                        redis::ConnectionInfo {
                            addr: redis::ConnectionAddr::Tcp(host.clone(), port.clone()),
                            redis: redis::RedisConnectionInfo {
                                db: db.clone() as i64,
                                username: username.clone(),
                                password: password.clone(),
                            },
                        })
                        .expect("invalid redis connector source describe")
                        .get_connection()
                        .expect("invalid connect redis"),
                },
                event_rx,
                disconnect_rx,
                event_time_duration: None,
            }
        }
    }

    fn handle_event(self: &mut Self, e: &event::BinderEvent) {
        match &e.binder_type {
            event::BinderEventType::Create {
                table_id,
                header_id,
                id,
                addr
            } => match &mut self.connector_type {
                ConnectorType::Redis { .. } => {
                    let b = types::Binder {
                        job_id: e.job_id.clone(),
                        binder_type: types::BinderType::Redis,
                        table_id: table_id.clone(),
                        header_id: header_id.to_string(),
                        id: id.clone(),
                        addr: addr.clone(),
                    };

                    self.binders.push(b)
                }
                ConnectorType::Tableflow { .. } => self.binders.push(
                    types::Binder {
                        job_id: e.job_id.clone(),
                        binder_type: types::BinderType::Tableflow { page: 0 },
                        table_id: table_id.clone(),
                        header_id: header_id.to_string(),
                        id: id.clone(),
                        addr: addr.clone(),
                    }
                )
            },
            event::BinderEventType::Stop => {
                match &mut self.connector_type {
                    ConnectorType::Redis { conn } => {
                        let topics = common::lists::filter_map(
                            &self.binders,
                            |binder| binder.job_id.eq(&e.job_id),
                            |binder| binder.get_topic(),
                        );

                        let _ = conn.as_pubsub()
                            .unsubscribe(topics);
                    }
                    _ => {}
                }

                common::lists::remove_if(&mut self.binders, |binder| binder.job_id.eq(&e.job_id));
            }
        }
    }

    pub async fn start(mut self) {
        let ref mut connector = self;
        let mut ticker = tokio::time::interval(
            connector
                .event_time_duration
                .unwrap_or(DEFAULT_EVENT_TIME_DURATION)
        );

        let (tx, mut rx) = mpsc::unbounded_channel::<event::ConnectorEvent>();

        loop {
            tokio::select! {
                Some(_) = connector.disconnect_rx.recv() => break,
                Some(events) = connector.event_rx.recv() => common::lists::for_each(&events, |e| connector.handle_event(e)),
                _ = ticker.tick(), if connector.is_tableflow() => {},
                Some(event) = rx.recv() => send_to_worker(&connector.binders, event),
                Some(event) = async {
                    match &mut connector.connector_type {
                        ConnectorType::Redis { conn } => {
                            println!("event fetched");
                            let mut pub_sub = conn.as_pubsub();
                            pub_sub.psubscribe(DATAFLOW_EVENT_PATTERN);
                            match pub_sub.get_message() {
                                Ok(msg) => match serde_json::from_slice::<event::ConnectorEvent>(msg.get_payload_bytes()) {
                                    Ok(event) => Some(event),
                                    Err(err) => {
                                        log::error!("invalid event payload: {:?}", err);
                                        None
                                    }
                                },
                                Err(err) => {
                                    log::error!("fail to get message: {:?}", err);
                                    None
                                }
                            }
                        },
                        _ => None
                    }
                } => {
                   let _ = tx.send(event)
                        .map_err(|err| {
                            log::error!("send event failed: {}", &err);
                            err
                        });
                }
                else => continue
            }
        }

        connector.disconnect_rx.close();
        connector.event_rx.close();
        rx.close();
    }

    fn is_tableflow(&self) -> bool {
        match &self.connector_type {
            ConnectorType::Tableflow { .. } => true,
            _ => false
        }
    }
}

fn send_to_worker(binders: &Vec<types::Binder>, event: event::ConnectorEvent) {
    let ref clients = common::lists::map(
        binders,
        |binder| dataflow_api::worker::new_dataflow_worker_client(
            dataflow_api::worker::DataflowWorkerConfig {
                host: None,
                port: None,
                uri: Some(binder.addr.clone()),
            }
        ),
    );

    let event_type = types::DataSourceEventType::from(&event.event_type);
    let event_time = time::SystemTime::now();

    common::lists::index_for_each(clients, |idx, cli| {
        let ref mut request = dataflow_api::dataflow_worker::ActionSubmitRequest::new();
        let b = &binders[idx];
        let target_key = event.get_key();

        if target_key == types::job_id(b.table_id.as_str(), b.header_id.as_str()) {
            let ref graph_event = event::GraphEvent::DataSourceEventSubmit(
                event::DataSourceEvent {
                    job_id: b.job_id.clone(),
                    to: b.id.clone(),
                    event_type: event_type.clone(),
                    data: event.entries.to_vec(),
                    event_time: event_time.clone(),
                }
            );

            let _ = serde_json::to_string(graph_event)
                .map_err(|err| err::CommonException::from(err))
                .and_then(|value| {
                    request.set_value(value.as_bytes().to_vec());
                    cli.submit_action(request)
                        .map(|resp| {
                            log::debug!("send action event success")
                        })
                        .map_err(|err| err::CommonException::from(err))
                })
                .map_err(|err| {
                    log::error!("serialize failed: {:?}", err);
                    err
                });
        }
    })
}
use std::collections;

use tokio::sync::mpsc;

use common::{event, types};
use common::err::ExecutionException;
use common::net::ClientConfig;
use stream::dataflow::{EventReceiver, EventSender, StreamConfig};

pub mod execution {
    use std::ops::Deref;
    use std::time;

    use actix::Actor;
    use serde::ser::SerializeStruct;
    use tokio::sync::mpsc;
    use common::{err, event, types};

    use crate::constants;
    use stream::dataflow as datastream;
    use stream::{pipeline, state, trigger, window};
    use common::types::{DataEventType, stream};
    use crate::sink::InternalSink;
    use proto::common::common as proto_common;
    use proto::common::stream as proto_stream;

    #[derive(Debug)]
    pub struct LocalExecutorManager {
        router: super::ExecutorRouter,
        pub job_id: proto_common::JobId,
    }

    impl super::Graph {
        pub fn new(ctx: types::DataflowContext) -> Self {
            Self {
                router: Default::default(),
                job_id: ctx.job_id.clone(),
            }
        }

        pub fn stop(&self) -> Result<(), err::ExecutionException> {
            for entry in &self.router {
                match exec.try_execute(event::DataEvent {
                    job_id: self.job_id.into(),
                    row_idx: 0,
                    to: entry.0.clone(),
                    event_type: DataEventType::STOP,
                    data: vec![],
                    old_data: vec![],
                    event_time: time::SystemTime::now(),
                    from: 0,
                }) {
                    Err(err) => {
                        log::error!("send event failed: {:?}", &err);
                        return Err(err::ExecutionException::fail_send_event_to_job_graph(&self.job_id));
                    }
                    _ => {}
                }
            }

            Ok(())
        }

        pub fn try_recv_data_events(&mut self, to: u64, events: Vec<event::DataEvent>) -> Result<(), err::ExecutionException> {
            match self.router.get(&to) {
                Some(exec) => match exec.try_recv_events(events) {
                    Ok(_) => Ok(()),
                    Err(err) =>
                        Err(err::ExecutionException {
                            kind: err::ErrorKind::SendGraphEventFailed,
                            msg: err.to_string(),
                        })
                },
                None => Ok(())
            }
        }
    }

    impl serde::Serialize for super::Graph {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
            let mut result = serializer.serialize_struct("Graph", 2)?;
            let _ = result.serialize_field("jobId", &self.job_id);
            let _ = result.serialize_field("nodes", &self.nodes);
            let _ = result.serialize_field("meta", &self.meta);
            result.end()
        }
    }

    #[derive(Debug)]
    pub enum NodeType {
        Local {
            op: stream::OperatorType,
            recipients: Vec<actix::Addr<Node>>,
        },
        Mirror(String),
    }

    #[derive(Debug)]
    pub struct Node {
        pub id: u64,
        pub node_type: NodeType,
        pub job_id: types::JobID,
        pub upstreams: Vec<types::NodeIdx>,
        datastream_close_tx: Option<mpsc::Sender<datastream::Close>>,
        datastream_tx: Option<datastream::EventSender<Vec<event::DataEvent>>>,
    }

    impl Node {
        fn id(&self) -> u64 {
            self.id.clone()
        }

        fn window_type(&self) -> window::WindowType {
            window::WindowType::Session {
                timeout: time::Duration::from_millis(100)
            }
        }

        fn trigger_type(&self) -> trigger::TriggerType {
            trigger::TriggerType::Watermark {
                firetime: time::Duration::from_millis(100)
            }
        }

        fn init_datastream(&mut self, recipients: Vec<actix::Addr<Node>>, operator: stream::OperatorType) {
            let (data_stream_tx, data_stream_rx) = datastream::new_event_pipe();
            let (close_tx, close_rx) = mpsc::channel(1);
            self.upstreams.sort();

            let event_pipeline = pipeline::OperatorEventPipeline::new(
                operator,
                self.job_id.clone(),
                self.id(),
                self.upstreams.clone(),
            );
            let data_stream = datastream::DataStream::new(
                self.window_type(),
                self.trigger_type(),
                data_stream_rx,
                close_rx,
                event_pipeline,
                InternalSink(self.job_id.clone(), recipients),
            );

            let _ = self.datastream_tx.insert(data_stream_tx);
            let _ = self.datastream_close_tx.insert(close_tx);

            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .build()
                .unwrap();

            let _ = rt.spawn(data_stream.start());
        }
    }

    impl Default for Node {
        fn default() -> Self {
            Node {
                id: 0,
                node_type: NodeType::Mirror(Default::default()),
                job_id: Default::default(),
                upstreams: vec![],
                datastream_close_tx: None,
                datastream_tx: None,
            }
        }
    }

    impl Actor for Node {
        type Context = actix::Context<Self>;

        fn started(&mut self, _ctx: &mut Self::Context) {
            match &self.node_type {
                NodeType::Local { recipients, op } =>
                    self.init_datastream(recipients.clone(), op.clone()),
                _ => {}
            }
        }
    }

    impl actix::Handler<event::EventSet<event::DataEvent, u64, types::ActionValue>> for Node {
        type Result = ();

        fn handle(&mut self,
                  msg: event::EventSet<event::DataEvent, u64, types::ActionValue>,
                  _ctx: &mut Self::Context) -> Self::Result {
            match &self.node_type {
                NodeType::Local { .. } => match &self.datastream_tx {
                    None => {}
                    Some(tx) => {
                        match tx.send(msg.events.clone()) {
                            Ok(_) => log::debug!("send event set {:?} to stream pipe success", &msg.events),
                            Err(err) =>
                                log::error!("send event set {:?} to stream pipe failed: {}", &msg.events, err)
                        }
                    }
                },
                NodeType::Mirror(addr) => {
                    let ref event_submit = event::GraphEvent::DataEvent {
                        job_id: self.job_id.clone(),
                        events: msg.events,
                        to: self.id,
                    };
                    super::send_to_worker(addr, event_submit)
                }
            }
        }
    }

    fn to_node(op: &types::OperatorInfo,
               job_id: types::JobID,
               local_hostname: &String,
               recipients: Vec<actix::Addr<Node>>) -> Node {
        if op.addr.starts_with(local_hostname) {
            return Node {
                id: op.id.clone(),
                node_type: NodeType::Local {
                    op: op.value.clone(),
                    recipients,
                },
                job_id,
                upstreams: op.upstream.to_vec(),
                datastream_close_tx: None,
                datastream_tx: None,
            };
        }

        Node {
            id: op.id.clone(),
            node_type: NodeType::Mirror(op.addr.clone()),
            job_id,
            upstreams: op.upstream.clone(),
            datastream_close_tx: None,
            datastream_tx: None,
        }
    }
}

pub type Graph = execution::LocalExecutorManager;

pub fn to_execution_graph(model: types::DataflowContext) -> Graph {
    Graph::new(model)
}

pub fn from_str(value: &str) -> Result<Graph, serde_json::Error> {
    let result = serde_json::from_str::<types::DataflowContext>(value);

    result.map(|model| to_execution_graph(model))
}

fn send_to_worker(addr: &String, event_submit: &event::GraphEvent) {}

pub type ExecutorRouter = collections::BTreeMap<u64, dyn Executor<event::DataEvent>>;

pub trait Executor<E> {
    fn try_recv_events(&self, events: Vec<E>) -> Result<(), ExecutionException>;
}

pub struct LocalExecutor<E> {
    pub executor_id: types::NodeIdx,

    sinks: Vec<EventSender<Vec<E>>>,
    source: EventReceiver<Vec<E>>,
    inner_sink: EventSender<Vec<E>>,

    pub config: StreamConfig,
}

impl<E> LocalExecutor<E> {
    pub fn with_inner_sink_and_config(sinks: Vec<EventSender<Vec<E>>>,
                                      source: EventReceiver<Vec<E>>,
                                      inner_sink: EventSender<Vec<E>>,
                                      config: StreamConfig) -> Self <E> {
        Self {
            sinks,
            source,
            inner_sink,
            config,
        }
    }
}

impl<E> Executor<E> for LocalExecutor<E> {
    fn try_recv_events(&self, events: Vec<E>) -> Result<(), ExecutionException> {
        todo!()
    }
}

pub struct RemoteExecutor<E> {
    source: EventReceiver<E>,

    pub config: ClientConfig,
}

impl<E> Executor<E> for RemoteExecutor<E> {
    fn try_recv_events(&self, events: Vec<E>) -> Result<(), ExecutionException> {
        todo!()
    }
}

pub trait SourceManager {
    fn
}
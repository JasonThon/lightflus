use std::collections;

use tokio::sync::mpsc;

use crate::{err, event, types};

pub mod execution {
    use std::ops::Deref;
    use std::time;

    use actix::Actor;
    use serde::ser::SerializeStruct;
    use tokio::sync::mpsc;

    use crate::{err, event, runtime, types};
    use crate::constants;
    use crate::stream as datastream;
    use crate::stream::{pipeline, state, trigger, window};
    use crate::types::formula;

    #[derive(Debug)]
    pub struct ExecutionGraph {
        pub job_id: types::JobID,
        pub meta: types::AdjacentList,
        pub nodes: types::NodeSet,
        addrmap: runtime::AddrMap,
    }

    impl super::Graph {
        pub fn new(job_id: types::JobID,
                   meta: types::AdjacentList,
                   nodes: types::NodeSet) -> Self {
            ExecutionGraph {
                job_id,
                meta,
                nodes,
                addrmap: Default::default(),
            }
        }

        pub fn build_dag(&mut self, job_id: types::JobID) {
            self.addrmap = build_addr_map(job_id, &self.meta, &self.nodes);
        }

        pub fn try_recv(&self, event: event::DataSourceEvent) -> Result<(), err::ExecutionException> {
            match self.addrmap
                .get(&event.to) {
                Some(addr) => addr
                    .try_send(event)
                    .map_err(|err|
                        err::ExecutionException {
                            kind: err::ErrorKind::ActorSendError,
                            msg: err.to_string(),
                        }
                    ),
                None => Ok(())
            }
        }

        pub fn stop(&self) -> Result<(), err::ExecutionException> {
            for (id, addr) in &self.addrmap {
                match addr.try_send(event::DataSourceEvent {
                    job_id: self.job_id.clone(),
                    to: id.clone(),
                    event_type: types::DataSourceEventType::Stop,
                    data: vec![],
                    old_data: vec![],
                    event_time: std::time::SystemTime::now(),
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

        pub fn try_send_formula_op_events(&mut self, to: u64, events: Vec<event::FormulaOpEvent>) -> Result<(), err::ExecutionException> {
            match self.addrmap.get_mut(&to) {
                Some(addr) => match addr.try_send(event::EventSet::new(events)) {
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

    struct ActixSink(Vec<actix::Addr<Node>>);

    impl datastream::Sink<Vec<event::FormulaOpEvent>> for ActixSink {
        fn sink(&self, output: Vec<event::FormulaOpEvent>) {
            common::lists::for_each(&self.0, |addr| {
                addr.do_send(event::EventSet::new(output.clone()))
            })
        }
    }

    #[derive(Debug)]
    pub enum NodeType {
        Local {
            op: formula::FormulaOp,
            recipients: Vec<actix::Addr<Node>>,
        },
        Mirror(String),
    }

    #[derive(Debug)]
    pub struct Node {
        pub id: u64,
        pub node_type: NodeType,
        pub job_id: types::JobID,
        closed: bool,
        datastream_close_tx: Option<mpsc::Sender<datastream::Close>>,
        datastream_tx: Option<datastream::StreamPipeSender<Vec<event::FormulaOpEvent>>>,
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

        fn init_datastream(&mut self, recipients: Vec<actix::Addr<Node>>, operator: types::formula::FormulaOp) {
            let (data_stream_tx, data_stream_rx) = datastream::stream_pipe();
            let (close_tx, close_rx) = mpsc::channel(1);

            let event_pipeline = pipeline::FormulaOpEventPipeline::new(
                operator,
                self.job_id.clone(),
                self.id(),
            );
            let data_stream = datastream::DataStream::new(
                self.window_type(),
                self.trigger_type(),
                data_stream_rx,
                close_rx,
                event_pipeline,
                ActixSink(recipients),
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
                closed: false,
                datastream_close_tx: None,
                datastream_tx: None,
            }
        }
    }

    impl actix::Actor for Node {
        type Context = actix::Context<Self>;

        fn started(&mut self, _ctx: &mut Self::Context) {
            match &self.node_type {
                NodeType::Local { recipients, op } =>
                    self.init_datastream(recipients.clone(), op.clone()),
                _ => {}
            }
        }
    }

    impl actix::Handler<event::DataSourceEvent> for Node {
        type Result = ();

        fn handle(&mut self, mut msg: event::DataSourceEvent, _ctx: &mut Self::Context) -> Self::Result {
            match &msg.event_type {
                types::DataSourceEventType::Stop => match &self.datastream_close_tx {
                    None => {}
                    Some(tx) => {
                        let _ = tx.send(datastream::Close);
                        self.closed = true;
                    }
                },
                _ => {}
            }

            if self.closed {
                match &self.node_type {
                    NodeType::Local { recipients, .. } => common::lists::for_each(
                        recipients,
                        |addr| addr.do_send(msg.clone()),
                    ),
                    _ => {}
                }

                return;
            }

            match &self.node_type {
                NodeType::Local { op, recipients } => match op {
                    types::formula::FormulaOp::Reference {
                        table_id,
                        header_id,
                    } => while let Some(tx) = &self.datastream_tx {
                        let client = data_client::new_data_engine_client(
                            data_client::DataEngineConfig {
                                host: None,
                                port: None,
                                uri: common::sysenv::get_env(constants::TABLEFLOW_URI_ENV_KEY),
                            }
                        );
                        let ref mut request = data_client::tableflow::QueryRequest::new();
                        request.set_tableId(table_id.clone());
                        request.set_headerId(header_id.clone());
                        let result = client.query(request);

                        match result {
                            Ok(resp) => {
                                let entries = common::lists::map(
                                    &resp.resultSet.to_vec(),
                                    |entry| types::Entry::from(entry),
                                );

                                msg.data = entries;
                            }
                            Err(err) =>
                                log::error!("query data failed: {}", err)
                        }

                        match tx.send(common::lists::map(
                            &msg.data,
                            |entry| event::FormulaOpEvent {
                                row_idx: entry.row_idx.clone(),
                                job_id: self.job_id.clone(),
                                data: entry.value.to_vec(),
                                old_data: common::lists::filter_map(
                                    &msg.old_data,
                                    |old| old.row_idx == entry.row_idx,
                                    |old| old.value.clone())
                                    .first()
                                    .unwrap_or(&vec![])
                                    .clone(),
                                from: self.id(),
                                action: msg.event_type
                                    .clone()
                                    .into(),
                                event_time: msg.event_time.clone(),
                            })
                        ) {
                            Ok(_) => log::debug!("event sent successfully"),
                            Err(err) =>
                                log::error!("event {:?} sent failed {}", &msg, err)
                        }
                    },

                    _ => log::error!("not reference formula op cannot receive datasource event: {:?}", &msg)
                },
                NodeType::Mirror(_) =>
                    log::error!("Mirror node does not support to handle datasource event: {:?}", &msg)
            }
        }
    }

    impl actix::Handler<event::EventSet<event::FormulaOpEvent, u64, types::ActionValue>> for Node {
        type Result = ();

        fn handle(&mut self,
                  msg: event::EventSet<event::FormulaOpEvent, u64, types::ActionValue>,
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
                    let ref event_submit = event::GraphEvent::FormulaOpEventSubmit {
                        job_id: self.job_id.clone(),
                        events: msg.events,
                        to: self.id,
                    };
                    super::send_to_worker(addr, event_submit)
                }
            }
        }
    }

    fn to_node(op: &types::Operator,
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
                closed: false,
                datastream_close_tx: None,
                datastream_tx: None,
            };
        }

        Node {
            id: op.id.clone(),
            node_type: NodeType::Mirror(op.addr.clone()),
            job_id,
            closed: false,
            datastream_close_tx: None,
            datastream_tx: None,
        }
    }

    pub fn build_addr_map(job_id: types::JobID,
                          meta: &types::AdjacentList,
                          nodes: &types::NodeSet) -> super::AddrMap {
        let ref local_hostname = common::hostname().expect("");

        let traversed = types::traverse_from_bottom(meta);
        let mut addrmap = super::AddrMap::new();

        for ref adj in traversed {
            let recipients = common::lists::map(
                &adj.neighbors,
                |neighbor_id| {
                    match addrmap.get(neighbor_id) {
                        Some(addr) => (*addr).clone(),
                        None => {
                            let operator = nodes
                                .get(&neighbor_id.to_string())
                                .unwrap();

                            addrmap.insert(
                                neighbor_id.clone(),
                                to_node(operator, job_id.clone(), local_hostname, vec![])
                                    .start(),
                            );

                            (*addrmap
                                .get(neighbor_id)
                                .unwrap()
                            ).clone()
                        }
                    }
                });

            addrmap.insert(
                adj.center.clone(),
                to_node(nodes.get(&adj.center.to_string())
                            .unwrap(),
                        job_id.clone(),
                        local_hostname,
                        recipients)
                    .start(),
            );
        }

        addrmap
    }
}

pub type Graph = execution::ExecutionGraph;

pub fn to_execution_graph(model: types::GraphModel) -> Graph {
    Graph::new(
        model.job_id,
        model.meta,
        model.nodes,
    )
}

pub fn from_str(value: &str) -> Result<Graph, serde_json::Error> {
    let result = serde_json::from_str::<types::GraphModel>(value);

    result.map(|model| to_execution_graph(model))
}

fn send_to_worker(addr: &String, event_submit: &event::GraphEvent) {
    let client = dataflow_api::worker::new_dataflow_worker_client(
        dataflow_api::worker::DataflowWorkerConfig {
            host: None,
            port: None,
            uri: Some(addr.clone()),
        }
    );

    let ref mut request = dataflow_api::dataflow_worker::ActionSubmitRequest::new();
    match serde_json::to_vec(event_submit) {
        Ok(data) => {
            request.set_value(data);
            match client.submit_action(request) {
                Ok(_) => log::debug!("submit action success"),
                Err(err) =>
                    log::error!("submit action {:?} failed {:?}", event_submit, err)
            }
        }
        Err(err) =>
            log::error!("serialize action {:?} failed: {:?}", event_submit, err)
    }
}

pub type AddrMap = collections::HashMap<u64, actix::Addr<execution::Node>>;

pub mod execution {
    use std::borrow;
    use std::cell;
    use std::collections;
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Formatter;
    use std::future::Future;

    use actix::{Actor, Addr, Recipient, Running};
    use petgraph::graph::NodeIndex;
    use petgraph::prelude::EdgeRef;
    use serde::{Deserializer, Serializer};
    use serde::de::{EnumAccess, Error, MapAccess, SeqAccess};
    use serde::ser::SerializeStruct;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::mpsc::UnboundedSender;
    use data_client::DataEngineConfig;
    use crate::event::{ConnectorEvent, Event, FormulaOpEvent, FormulaOpEventType};

    use crate::graph::worker::new_worker;
    use crate::runtime::execution::Node::{Local, Ref};
    use crate::types::formula::FormulaOp;
    use crate::types::SourceDesc;
    use crate::{err, event, types, conn};
    use crate::types::{AdjacentVec, formula, JobID};

    pub struct ExecutionGraph {
        pub job_id: types::JobID,
        pub meta: Vec<AdjacentVec>,
        pub nodes: collections::BTreeMap<u64, Operator>,
        graphmap: petgraph::Graph<Node, actix::Recipient<FormulaOpEvent>>,
        node_addr_map: collections::HashMap<u64, actix::Addr<Node>>,
    }

    impl super::Graph {
        pub(crate) fn remove_nodes(&self) -> Result<(), err::ExecutionException> {
            todo!()
        }

        pub fn new(job_id: types::JobID,
                   meta: Vec<AdjacentVec>,
                   mut nodes: collections::BTreeMap<u64, Operator>,
                   dag_required: bool) -> Self {
            if dag_required {
                ExecutionGraph {
                    job_id: job_id.clone(),
                    meta,
                    nodes,
                    graphmap: build_graph(job_id, &meta, &nodes),
                    node_addr_map: Default::default(),
                }
            } else {
                ExecutionGraph {
                    job_id,
                    meta,
                    nodes,
                    graphmap: Default::default(),
                    node_addr_map: Default::default(),
                }
            }
        }

        pub(crate) fn dispatch(&self) -> Result<(), err::ExecutionException> {
            todo!()
        }

        pub(crate) fn stop(&self) -> Result<(), err::ExecutionException> {
            todo!()
        }

        pub fn build_dag(&mut self, job_id: JobID) {
            self.graphmap = build_graph(job_id, &self.meta, &self.nodes);
        }

        pub fn try_recv(&mut self, event: FormulaOpEvent) -> Result<(), err::ExecutionException> {
            match self.node_addr_map
                .get_mut(&event.to) {
                Some(addr) => addr
                    .try_send(event)
                    .map_err(|err| err::ExecutionException::from(err)),
                None => Ok(())
            }
        }

        pub fn start(&mut self) {
            let mut wait_queue = collections::HashSet::new();

            self.graphmap.node_indices()
                .for_each(|idx| {
                    let edges = self.graphmap.edges(idx.clone());

                    edges.for_each(|edge| {
                        let target_node_idx = edge.target();
                        let target_node = self.graphmap.node_weight(target_node_idx.clone()).unwrap();
                        let target_id = target_node.id();

                        self.graphmap.remove_edge(edge.id());
                        match self.node_addr_map.get(&target_id) {
                            None => {
                                let addr = target_node.start();
                                let recipient = addr.recipient();
                                self.node_addr_map.insert(target_id, addr);

                                self.graphmap.add_edge(idx.clone(), target_node_idx, recipient);
                            }
                            Some(addr) => self.graphmap
                                .add_edge(idx.clone(), target_node_idx, addr.recipient())
                        }

                        wait_queue.remove(&target_node_idx);
                        wait_queue.insert(idx);
                    })
                });

            wait_queue.iter()
                .for_each(|idx| {
                    let node = self.graphmap
                        .node_weight(idx.clone())
                        .unwrap();
                    self.node_addr_map.insert(node.id(), node.start());
                });

            wait_queue.clear();
        }

        fn start_node(&mut self, node_index: NodeIndex) -> Option<actix::Addr<Node>> {
            self.graphmap.node_weight(node_index)
                .map(|node| node.start())
        }
    }

    impl serde::Serialize for super::Graph {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
            let mut result = serializer.serialize_struct("Graph", 2)?;
            result.serialize_field("jobId", &self.job_id);
            result.serialize_field("nodes", &self.nodes);
            result.serialize_field("meta", &self.meta);
            result.end()
        }
    }

    impl<'de> serde::Deserialize<'de> for super::Graph {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            struct GraphVisitor;

            impl<'de> serde::de::Visitor<'de> for GraphVisitor {
                type Value = super::Graph;

                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    formatter.write_str("start graph key visiting")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
                    todo!()
                }

                fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                    todo!()
                }
            }

            todo!()
        }
    }

    #[derive(Clone)]
    enum Node {
        Local {
            op: formula::FormulaOp,
            id: u64,
            recipients: Vec<actix::Recipient<event::FormulaOpEvent>>,
            handlers: Vec<tokio::task::JoinHandle<()>>,
        },
        Ref {
            addr: String,
            id: u64,
            job_id: JobID,
        },
    }

    impl Node {
        pub(crate) fn id(&self) -> u64 {
            match self {
                Local {
                    id, ..
                } => id.clone(),
                Ref {
                    id, ..
                } => id.clone()
            }
        }

        pub(crate) fn add_recipient(&mut self, other: Recipient<FormulaOpEvent>) {
            match self {
                Local {
                    recipients, ..
                } => recipients.push(other),
                _ => {}
            }
        }

        fn convert_to_formula_op_event(&self, event_type: FormulaOpEventType, event: ConnectorEvent) -> FormulaOpEvent {
            FormulaOpEvent {
                job_id: Default::default(),
                from: self.id(),
                to: 0,
                event_type,
                data: event.get_value().1,
                event_time: event.event_time(),
            }
        }
    }

    impl actix::Actor for Node {
        type Context = actix::Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            match self {
                Local {
                    op,
                    recipients,
                    handlers, ..
                } => {
                    match op {
                        formula::FormulaOp::Reference {
                            table_id,
                            header_id,
                            sources
                        } => {
                            let (ref tx, mut rx) = mpsc::unbounded_channel();
                            let ref mut connector_handlers = core::lists::map(sources, |src| {
                                let connector = conn::to_connector(src, table_id, header_id, tx);
                                tokio::spawn(connector)
                            });

                            let sender_handler = tokio::spawn(async {
                                loop {
                                    match rx.try_recv() {
                                        Ok(msg) => {
                                            let op_event = self.convert_to_formula_op_event(FormulaOpEventType::Reference, msg);

                                            core::lists::for_each(recipients, |receiver| {
                                                receiver.try_send(op_event.clone());
                                            })
                                        }
                                        Err(err) => {}
                                    }
                                }
                            });

                            handlers.append(connector_handlers);
                            handlers.push(sender_handler)
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
            todo!()
        }
    }

    impl actix::Handler<FormulaOpEvent> for Node {
        type Result = ();

        fn handle(&mut self, msg: FormulaOpEvent, ctx: &mut Self::Context) -> Self::Result {
            todo!()
        }
    }

    #[derive(Clone, serde::Serialize, serde::Deserialize)]
    pub struct Operator {
        pub addr: String,
        pub value: formula::FormulaOp,
        pub id: u64,
    }

    impl Operator {
        pub(crate) fn is_same_host(&self, ip: &String) -> bool {
            self.addr.eq(ip)
        }

        pub(crate) fn to_node(&self, job_id: JobID, local_addr: &String) -> Node {
            if self.addr.eq(local_addr) {
                Local {
                    op: self.value.clone(),
                    id: self.id.clone(),
                    recipients: vec![],
                    handlers: vec![],
                }
            }

            Ref {
                addr: self.addr.clone(),
                id: self.id.clone(),
                job_id,
            }
        }
    }

    fn build_graph(job_id: JobID, meta: &Vec<AdjacentVec>, mut nodes: &BTreeMap<u64, Operator>) -> petgraph::Graph<Node, actix::Recipient<FormulaOpEvent>> {
        let local_addr = core::local_ip().expect("");
        let mut graphmap = petgraph::Graph::new();

        core::lists::for_each(&meta, |adj| {
            let option = nodes.get_mut(&adj.center);

            if option.is_some() {
                let op = option.unwrap();

                let central = graphmap.add_node(op.to_node(job_id.clone(), &local_addr));

                let ref neighbors = core::lists::map(&adj.neighbors, |neigh_id|
                    graphmap.add_node(nodes.get(neigh_id)
                        .unwrap()
                        .to_node(job_id.clone(), &local_addr)),
                );

                graphmap.extend_with_edges(
                    core::lists::map(
                        neighbors,
                        |idx| (central.clone(), idx))
                        .as_slice()
                );
            }
        });

        graphmap
    }
}

pub type Graph = execution::ExecutionGraph;

pub mod formula {
    use crate::types::formula;

    impl formula::FormulaOp {}
}

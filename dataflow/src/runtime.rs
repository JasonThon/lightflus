use crate::types;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AdjacentVec {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub neighbors: Vec<u64>,
    pub center: u64,
}

pub mod formula {
    use std::collections;

    use crate::{graph, types};
    use crate::source;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct InitState {
        pub(crate) page: u64,
        pub(crate) limit: u64,
        pub(crate) done: bool,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
    pub struct FormulaGraph {
        pub meta: Vec<super::AdjacentVec>,
        pub data: collections::BTreeMap<u64, FormulaOp>,
    }

    #[serde(tag = "type")]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum FormulaOp {
        Reference {
            #[serde(rename(serialize = "tableId"))]
            table_id: String,
            #[serde(rename(serialize = "headerId"))]
            header_id: String,
            sources: Vec<source::SourceDesc>,
        },
        Add,
    }

    const REFERENCE_OP: &'static str = "Reference";

    impl core::KeyedValue<String, FormulaOp> for FormulaOp {
        fn key(&self) -> String {
            match self {
                FormulaOp::Reference { .. } => REFERENCE_OP.to_string(),
                _ => "".to_string()
            }
        }

        fn value(&self) -> FormulaOp {
            self.clone()
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, actix::Message)]
    pub struct FormulaOpEvent {
        pub job_id: types::JobID,
        pub from: u64,
        pub to: u64,
        #[serde(rename(serialize = "eventType"))]
        pub event_type: FormulaOpEventType,
        pub data: Vec<types::RowData>,
        pub event_time: std::time::SystemTime,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub enum FormulaOpEventType {
        SinkResult,
        Reference,
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecutionException {
    pub kind: ErrorKind,
    pub msg: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ErrorKind {}


pub mod execution {
    use std::borrow;
    use std::cell;
    use std::collections;
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Formatter;
    use std::process::id;

    use actix::{Actor, Addr};
    use petgraph::graph::NodeIndex;
    use serde::{Deserializer, Serializer};
    use serde::de::{EnumAccess, Error, MapAccess, SeqAccess};
    use serde::ser::SerializeStruct;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::UnboundedSender;
    use data_client::DataEngineConfig;

    use crate::graph::worker::new_worker;
    use crate::runtime::{AdjacentVec, formula, Graph};
    use crate::runtime::execution::Node::{Local, Ref};
    use crate::runtime::formula::{FormulaOp, FormulaOpEvent};
    use crate::source::SourceDesc;
    use crate::types;
    use crate::types::JobID;

    pub struct ExecutionGraph {
        pub job_id: types::JobID,
        pub meta: Vec<super::AdjacentVec>,
        pub nodes: collections::BTreeMap<u64, Operator>,
        graphmap: petgraph::Graph<Node, bool>,
        node_addr_map: collections::BTreeMap<u64, actix::Addr<Node>>,
    }

    impl super::Graph {
        pub(crate) fn remove_nodes(&self) -> Result<(), super::ExecutionException> {
            todo!()
        }

        pub fn new(job_id: types::JobID,
                   meta: Vec<super::AdjacentVec>,
                   mut nodes: collections::BTreeMap<u64, Operator>,
                   dag_required: bool) -> Self {
            if dag_required {
                ExecutionGraph {
                    job_id,
                    meta,
                    nodes,
                    graphmap: build_graph(&meta, &nodes),
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

        pub(crate) fn dispatch(&self) -> Result<(), super::ExecutionException> {
            todo!()
        }

        pub(crate) fn stop(&self) -> Result<(), super::ExecutionException> {
            todo!()
        }

        pub fn build_dag(&mut self) {
            self.graphmap = build_graph(&self.meta, &self.nodes);
        }

        pub fn start(&mut self) {
            let mut startflag = collections::HashSet::new();
            self.nodes.clear();
            self.meta.clear();

            self.graphmap.edge_indices()
                .for_each(|idx| {
                    let endpoints_options = self.graphmap.edge_endpoints(idx);
                    if endpoints_options.is_some() {
                        let endpoints = endpoints_options.unwrap();
                        let source_node = endpoints.0;
                        let target_node = endpoints.1;

                        let source_index = source_node.index();
                        if !startflag.contains(&source_index) {
                            self.start_node(source_node)
                        }
                        startflag.insert(source_index);

                        let target_index = target_node.index();
                        if !startflag.contains(&target_index) {
                            self.start_node(target_node);
                        }

                        startflag.insert(target_index);
                    }
                });
        }

        fn start_node(&mut self, node_index: NodeIndex) {
            let node_option = self.graphmap.node_weight(node_index);

            if node_option.is_some() {
                let node = node_option.unwrap();
                self.node_addr_map.insert(node.id(), node.start());
            }
        }
    }

    impl serde::Serialize for super::Graph {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
            let mut result = serializer.serialize_struct("Graph", 2)?;
            result.serialize_field("jobId", &self.job_id);
            result.serialize_field("nodes", &self.nodes);
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
            receipants: Vec<actix::Recipient<formula::FormulaOpEvent>>,
        },
        Ref {
            addr: String,
            id: u64,
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
    }

    impl actix::Actor for Node {
        type Context = actix::Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            match self {
                Local {
                    op, receipants, ..
                } => {
                    match op {
                        formula::FormulaOp::Reference {
                            table_id,
                            header_id,
                            sources
                        } => {
                            let conns = core::lists::map(sources, |src| src.to_connector());


                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    impl actix::Handler<formula::FormulaOpEvent> for Node {
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

        pub(crate) fn to_node(&self, local_addr: &String) -> Node {
            if self.addr.eq(local_addr) {
                Local {
                    op: self.value.clone(),
                    id: self.id.clone(),
                    receipants: vec![],
                }
            }

            Ref {
                addr: self.addr.clone(),
                id: self.id.clone(),
            }
        }
    }

    fn build_graph(meta: &Vec<AdjacentVec>, mut nodes: &BTreeMap<u64, Operator>) -> petgraph::Graph<Node, bool> {
        let local_addr = core::local_ip().expect("");
        let mut graphmap = petgraph::Graph::new();

        core::lists::for_each(&meta, |adj| {
            let option = nodes.get_mut(&adj.center);

            if option.is_some() {
                let op = option.unwrap();

                let central = graphmap.add_node(op.to_node(&local_addr));

                let ref neighbors = core::lists::map(&adj.neighbors, |neigh_id|
                    graphmap.add_node(nodes.get(neigh_id)
                        .unwrap()
                        .to_node(&local_addr)),
                );

                graphmap.extend_with_edges(
                    core::lists::map(
                        neighbors,
                        |idx| (central, idx))
                        .as_slice()
                );
            }
        });

        graphmap
    }
}

pub type Graph = execution::ExecutionGraph;
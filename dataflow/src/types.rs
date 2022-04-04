use std::collections;

use crate::{err, types};
use crate::event;

pub type AdjacentList = Vec<AdjacentVec>;

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobID {
    pub table_id: String,
    pub header_id: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub row_idx: u64,
    pub value: Vec<u8>,
}

impl From<&data_client::tableflow::Entry> for Entry {
    fn from(entry: &data_client::tableflow::Entry) -> Self {
        Self {
            row_idx: entry.get_rowIdx(),
            value: entry.get_value().to_vec(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct AdjacentVec {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub neighbors: Vec<u64>,
    pub center: u64,
}

pub mod formula {
    use std::collections;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct InitState {
        pub(crate) page: u64,
        pub(crate) limit: u64,
        pub(crate) done: bool,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
    pub struct FormulaGraph {
        #[serde(default)]
        pub meta: Vec<super::AdjacentVec>,
        #[serde(default)]
        pub data: collections::BTreeMap<u64, FormulaOp>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    #[serde(tag = "type")]
    pub enum FormulaOp {
        Reference {
            #[serde(rename(serialize = "tableId"))]
            table_id: String,
            #[serde(rename(serialize = "headerId"))]
            header_id: String,
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
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SourceDesc {
    Redis {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        db: usize,
    },
    Tableflow {
        host: String,
        port: usize,
        limit: u32,
        event_time: Option<u64>,
    },
}

pub fn job_id(table_id: &str, header_id: &str) -> JobID {
    JobID {
        table_id: table_id.to_string(),
        header_id: header_id.to_string(),
    }
}

pub type AddrMap = collections::HashMap<u64, actix::Recipient<event::FormulaOpEvent>>;

pub fn traverse_from_bottom(meta: &Vec<AdjacentVec>) -> Vec<AdjacentVec> {
    let mut results = vec![];

    let mut grouped = core::lists::group_hashmap(meta, |adj| adj.center.clone());

    core::lists::for_each(meta, |adj| {
        let mut flag = false;
        for id in &adj.neighbors {
            if grouped.contains_key(id) {
                flag = true;
                break;
            }
        }

        if !flag {
            grouped.remove(&adj.center);
            results.push(adj.clone());
        }
    });

    for (_, value) in grouped {
        results.push(value.clone())
    }

    results
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct Operator {
    pub addr: String,
    pub value: formula::FormulaOp,
    pub id: u64,
}

pub type NodeSet = collections::BTreeMap<u64, Operator>;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphModel {
    pub job_id: JobID,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub meta: AdjacentList,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    #[serde(default)]
    pub nodes: NodeSet,
}

impl GraphModel {
    pub(crate) fn dispatch(&self) -> Result<(), err::CommonException> {
        let mut group = collections::HashMap::<String, Vec<&Operator>>::new();

        for (_, operator) in &self.nodes {
            match group.get(&operator.addr) {
                Some(ops) => {
                    let mut new_operators = vec![operator];
                    new_operators.extend(ops);
                    group.insert(operator.addr.clone(), new_operators);
                }
                None => {
                    group.insert(operator.addr.clone(), vec![operator]);
                }
            }
        }

        for (addr, ops) in group {
            let client = dataflow_api::worker::new_dataflow_worker_client(
                dataflow_api::worker::DataflowWorkerConfig {
                    host: None,
                    port: None,
                    uri: Some(addr),
                }
            );

            let ref graph_event = event::GraphEvent::ExecutionGraphSubmit {
                ops: types::GraphModel::from(ops),
                job_id: self.job_id.clone(),
            };

            let ref mut request = dataflow_api::dataflow_worker::ActionSubmitRequest::default();
            let result = serde_json::to_vec(graph_event)
                .map_err(|err| err::CommonException::from(err))
                .and_then(|value| {
                    request.set_value(value);
                    client.submit_action(request)
                        .map_err(|err| err::CommonException::from(err))
                })
                .map(|resp| {
                    log::info!("submit success")
                });

            if result.is_err() {
                return result;
            }
        }
        Ok(())
    }

    pub(crate) fn new(job_id: JobID,
                      meta: AdjacentList,
                      nodes: NodeSet) -> GraphModel {
        GraphModel {
            job_id,
            meta,
            nodes,
        }
    }
}

impl From<Vec<&Operator>> for GraphModel {
    fn from(ops: Vec<&Operator>) -> Self {
        todo!()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum BinderType {
    Tableflow {
        page: u32
    },
    Redis,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Binder {
    pub job_id: JobID,
    pub binder_type: BinderType,
    pub table_id: String,
    pub header_id: String,
    pub id: u64,
    pub addr: String,
}

impl Binder {
    pub(crate) fn get_topic(&self) -> String {
        format!("{}:{}", &self.table_id, &self.header_id)
    }
}

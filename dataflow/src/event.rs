use std::collections;

use crate::types;
use crate::types::JobID;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct BinderEvent {
    pub job_id: types::JobID,
    pub binder_type: BinderEventType,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum BinderEventType {
    Create {
        table_id: String,
        header_id: String,
        id: u64,
        addr: String,
    },
    Stop,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TableAction {
    FormulaUpdate {
        table_id: String,
        header_id: String,
        graph: types::formula::FormulaGraph,
    },
    TableSubmission {
        table_id: String,
        data: collections::BTreeMap<String, Vec<u8>>,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct TableEvent {
    action: TableAction,
    event_time: std::time::SystemTime,
}

impl TableEvent {
    pub fn action(&self) -> &TableAction {
        &self.action
    }
    pub fn new(action: TableAction) -> TableEvent {
        TableEvent {
            action,
            event_time: std::time::SystemTime::now(),
        }
    }
}

impl Event<String, TableAction> for TableEvent {
    fn event_time(&self) -> std::time::SystemTime {
        self.event_time.clone()
    }

    fn get_key(&self) -> String {
        todo!()
    }

    fn get_value(&self) -> TableAction {
        todo!()
    }
}


pub trait Event<K, V> {
    fn event_time(&self) -> std::time::SystemTime;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEvent {
    pub event_type: ConnectorEventType,
    pub table_id: String,
    pub header_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<types::Entry>,
    pub timestamp: std::time::SystemTime,
    pub binders: Vec<types::Binder>,
}

impl ConnectorEvent {
    pub(crate) fn to_formula_op_event_type(&self) -> FormulaOpEventType {
        match &self.event_type {
            ConnectorEventType::Tableflow { page, limit } =>
                FormulaOpEventType::TableflowTrigger {
                    page: page.clone(),
                    limit: limit.clone(),
                },
            ConnectorEventType::Action(action_type) => {
                match action_type.clone() {
                    DELETE => FormulaOpEventType::Delete,
                    UPDATE => FormulaOpEventType::Update,
                    INSERT => FormulaOpEventType::Insert,
                    _ => FormulaOpEventType::Invalid
                }
            }
            ConnectorEventType::Close => FormulaOpEventType::Invalid
        }
    }
}

impl Event<JobID, (ConnectorEventType, Vec<types::Entry>)> for ConnectorEvent {
    fn event_time(&self) -> std::time::SystemTime {
        self.timestamp.clone()
    }

    fn get_key(&self) -> JobID {
        types::job_id(self.table_id.as_str(), self.table_id.as_str())
    }

    fn get_value(&self) -> (ConnectorEventType, Vec<types::Entry>) {
        (self.event_type.clone(), self.entries.to_vec())
    }
}

#[derive(PartialEq, Clone, Default)]
pub struct WrappedQueryResponse {
    resp: data_client::tableflow::QueryResponse,
    id: types::JobID,
}

pub fn new_wrapped_query_resp(resp: data_client::tableflow::QueryResponse, id: types::JobID) -> WrappedQueryResponse {
    WrappedQueryResponse {
        resp,
        id,
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ConnectorEventType {
    Tableflow {
        page: u32,
        limit: u32,
    },
    Action(ActionType),
    Close,
}

pub type ActionType = usize;

pub const INSERT: usize = 0;
pub const UPDATE: usize = 1;
pub const DELETE: usize = 2;

#[derive(Clone, serde::Serialize, serde::Deserialize, actix::Message, Debug)]
#[rtype(result = "()")]
pub struct FormulaOpEvent {
    pub job_id: types::JobID,
    pub from: u64,
    pub to: u64,
    #[serde(rename(serialize = "eventType"))]
    pub event_type: FormulaOpEventType,
    pub data: Vec<types::Entry>,
    pub event_time: std::time::SystemTime,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum FormulaOpEventType {
    TableflowTrigger {
        page: u32,
        limit: u32,
    },
    Delete,
    Update,
    Insert,
    Stop,
    Invalid,
}

impl From<&ConnectorEventType> for FormulaOpEventType {
    fn from(_: &ConnectorEventType) -> Self {
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GraphEvent {
    ExecutionGraphSubmit {
        ops: types::GraphModel,
        job_id: types::JobID,
    },
    NodeEventSubmit(FormulaOpEvent),
    StopGraph {
        job_id: types::JobID,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Disconnect;
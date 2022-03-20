use std::collections::BTreeMap;

use crate::types;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum TableAction {
    HeaderSubmit {
        data: Vec<String>,
        table_id: String,
        header_id: String,
    },
    FormulaUpdate {
        table_id: String,
        header_id: String,
        graph: types::formula::FormulaGraph,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TableEvent {
    action: TableAction,
    event_time: std::time::SystemTime,
}

impl TableEvent {
    pub fn action(&self) -> &TableAction {
        &self.action
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
}

impl Event<String, (ConnectorEventType, Vec<types::Entry>)> for ConnectorEvent {
    fn event_time(&self) -> std::time::SystemTime {
        self.timestamp.clone()
    }

    fn get_key(&self) -> String {
        format!("{}/{}", &self.table_id, &self.header_id)
    }

    fn get_value(&self) -> (ConnectorEventType, Vec<types::Entry>) {
        (self.event_type.clone(), self.entries.to_vec())
    }
}

#[derive(Clone)]
pub enum ConnectorEventType {
    Tableflow,
    Action(ActionType),
}

pub type ActionType = usize;

pub const INSERT: usize = 0;
pub const UPDATE: usize = 1;
pub const DELETE: usize = 2;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, actix::Message)]
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
    Reference,
}

use std::{collections, marker};

use bytes::Buf;

use crate::types;

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
#[serde(tag = "type")]
pub enum TableAction {
    #[serde(rename_all = "camelCase")]
    FormulaSubmit {
        table_id: String,
        header_id: String,
        graph: types::formula::FormulaGraph,
    },
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TableEvent {
    action: TableAction,
    event_time: String,
}

impl TableEvent {
    pub fn action(&self) -> &TableAction {
        &self.action
    }
    pub fn new(action: TableAction) -> TableEvent {
        TableEvent {
            action,
            event_time: format!("{:?}", chrono::Utc::now()),
        }
    }
}

impl Event<types::JobID, TableAction> for TableEvent {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.event_time
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
    }

    fn get_key(&self) -> types::JobID {
        match &self.action {
            TableAction::FormulaSubmit { table_id, header_id, .. } =>
                types::job_id(table_id.as_str(), header_id.as_str())
        }
    }

    fn get_value(&self) -> TableAction {
        self.action.clone()
    }
}


pub trait Event<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}


#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEvent {
    pub event_type: types::ConnectorEventType,
    pub table_id: String,
    pub header_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub entries: Vec<types::Entry>,
    pub old_values: Vec<types::Entry>,
    pub timestamp: String,
}

impl Event<types::JobID, (types::ConnectorEventType, Vec<types::Entry>)> for ConnectorEvent {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
            .as_str()
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap()
    }

    fn get_key(&self) -> types::JobID {
        types::job_id(self.table_id.as_str(), self.header_id.as_str())
    }

    fn get_value(&self) -> (types::ConnectorEventType, Vec<types::Entry>) {
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

#[derive(Clone, serde::Serialize, serde::Deserialize, actix::Message, Debug)]
#[rtype(result = "()")]
pub struct DataSourceEvent {
    #[serde(rename(serialize = "jobId", deserialize = "jobId"))]
    pub job_id: types::JobID,
    pub to: u64,
    #[serde(rename(serialize = "eventType", deserialize = "eventType"))]
    pub event_type: types::DataSourceEventType,
    pub data: Vec<types::Entry>,
    #[serde(rename(serialize = "oldData", deserialize = "oldData"))]
    pub old_data: Vec<types::Entry>,
    #[serde(rename(serialize = "eventTime", deserialize = "eventTime"))]
    pub event_time: std::time::SystemTime,
}

#[derive(serde::Serialize, serde::Deserialize, actix::Message, Debug)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum GraphEvent {
    #[serde(rename_all = "camelCase")]
    ExecutionGraphSubmit {
        ops: types::GraphModel,
        job_id: types::JobID,
    },
    DataSourceEventSubmit(DataSourceEvent),
    #[serde(rename_all = "camelCase")]
    TerminateGraph {
        job_id: types::JobID,
    },
    FormulaOpEventSubmit {
        job_id: types::JobID,
        events: Vec<FormulaOpEvent>,
        to: u64,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Disconnect;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FormulaOpEvent {
    pub row_idx: u64,
    pub job_id: types::JobID,
    pub data: Vec<u8>,
    pub old_data: Vec<u8>,
    pub from: u64,
    pub action: types::ActionType,
    pub event_time: std::time::SystemTime,
}

impl Event<u64, types::ActionValue> for FormulaOpEvent {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from(self.event_time)
    }

    fn get_key(&self) -> u64 {
        self.row_idx
    }

    fn get_value(&self) -> types::ActionValue {
        types::ActionValue {
            action: self.action.clone(),
            value: types::TypedValue::from(&self.data),
            from: self.from,
        }
    }
}

pub struct EventSet<T: Event<K, V>, K, V> {
    pub events: Vec<T>,
    phantom_key: marker::PhantomData<K>,
    phantom_value: marker::PhantomData<V>,
}

impl<T: Event<K, V>, K, V> EventSet<T, K, V> {
    pub fn new(events: Vec<T>) -> EventSet<T, K, V> {
        EventSet {
            events,
            phantom_key: Default::default(),
            phantom_value: Default::default(),
        }
    }
}

impl actix::Message for EventSet<FormulaOpEvent, u64, types::ActionValue> {
    type Result = ();
}
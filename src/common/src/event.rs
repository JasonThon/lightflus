use std::{collections, marker};

use bytes::Buf;
use proto::common::common::JobId;
use stream::dataflow;

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

pub trait KeyedEvent<K, V> {
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

impl KeyedEvent<types::JobID, (types::ConnectorEventType, Vec<types::Entry>)> for ConnectorEvent {
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
    DataSource(DataSourceEvent),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Disconnect;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalEvent {
    RowChangeStream(Vec<RowDataEvent>),
    Terminate {
        job_id: JobId,
        to: types::SinkId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowDataEvent {
    pub row_idx: types::RowIdx,
    pub job_id: types::JobID,
    pub to: types::SinkId,
    pub data: Vec<u8>,
    pub old_data: Vec<u8>,
    pub from: types::SourceId,
    pub event_type: types::DataEventType,
    pub event_time: std::time::SystemTime,
}

impl KeyedEvent<types::RowIdx, types::ActionValue> for LocalEvent {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from(self.event_time)
    }

    fn get_key(&self) -> types::RowIdx {
        self.row_idx
    }

    fn get_value(&self) -> types::ActionValue {
        types::ActionValue {
            action: self.event_type.clone(),
            value: types::TypedValue::from(&self.data),
            from: self.from,
        }
    }
}


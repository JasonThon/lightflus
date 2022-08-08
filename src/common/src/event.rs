use std::{collections, marker};
use std::fmt::{Display, Formatter};

use bytes::Buf;
use proto::common::common::JobId;
use proto::common::event::DataEvent;

use crate::types;

pub trait KeyedEvent<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}

#[derive(Clone, Debug)]
pub enum LocalEvent {
    RowChangeStream(Vec<RowDataEvent>),
    Terminate {
        job_id: JobId,
        to: types::SinkId,
    },
}

impl From<&DataEvent> for RowDataEvent {
    fn from(event: &DataEvent) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct RowDataEvent {
    pub row_idx: types::RowIdx,
    pub job_id: JobId,
    pub to: types::SinkId,
    pub data: Vec<u8>,
    pub old_data: Vec<u8>,
    pub from: types::SourceId,
    pub event_type: types::DataEventType,
    pub event_time: std::time::SystemTime,
}
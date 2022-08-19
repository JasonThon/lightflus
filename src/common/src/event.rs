use chrono::Timelike;
use proto::common::common::JobId;
use proto::common::event::{DataEvent, DataEventTypeEnum};
use proto::common::table::Entry;

use crate::types;

pub trait KeyedEvent<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}

#[derive(Clone, Debug)]
pub enum LocalEvent {
    RowChangeStream(Vec<RowDataEvent>),
    Terminate { job_id: JobId, to: types::SinkId },
}

impl From<&DataEvent> for RowDataEvent {
    fn from(event: &DataEvent) -> Self {
        // let event_time = event.get_event_time();
        // let datetime = chrono::Utc::now()
        //     .with_second(event_time.get_seconds() as u32)
        //     .and_then(|utc| utc.with_nanosecond(event_time.get_nanos() as u32))
        //     .unwrap();

        Self {
            job_id: event.get_job_id().clone(),
            to: event.get_to_operator_id(),
            data: event.get_data().to_vec(),
            old_data: event.get_old_data().to_vec(),
            from: event.get_from_operator_id(),
            event_type: event.get_event_type(),
            event_time: std::time::SystemTime::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RowDataEvent {
    pub job_id: JobId,
    pub to: types::SinkId,
    pub data: Vec<Entry>,
    pub old_data: Vec<Entry>,
    pub from: types::SourceId,
    pub event_type: DataEventTypeEnum,
    pub event_time: std::time::SystemTime,
}

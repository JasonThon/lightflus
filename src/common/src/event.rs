use proto::common::common::ResourceId;
use proto::common::event::KeyedDataEvent;

use crate::types;

pub trait KeyedEvent<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}

#[derive(Clone, Debug)]
pub enum LocalEvent {
    Terminate { job_id: ResourceId, to: types::SinkId },
    KeyedDataStreamEvent(KeyedDataEvent),
}

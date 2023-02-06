use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::time::SystemTime;

use prost::Message;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::kafka::KafkaMessage;
use crate::types::{self, TypedValue};

pub trait KeyedEvent<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}

/// The trait that is a stream event
/// A [`StreamEvent`] can be serialized to be sent to external sink (e.g. Kafka, Database).
pub trait StreamEvent: Send + Serialize {
    fn generate_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError>;
    fn from_slice(slice: &[u8]) -> Self;
    fn event_id(&self) -> i64;
    fn event_time(&self) -> i64;
}

/// [`LocalEvent`] can be transferred between threads safely by channel.
/// [`LocalEvent`] also can be serialized and sent through pipe or network to another process.
/// # Example
///
/// ```
/// use common::event::LocalEvent;
/// use proto::common::ResourceId;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, mut rx) = tokio::sync::mpsc::channel(10);
///     let _ = tx.send(LocalEvent::Terminate {
///         job_id: ResourceId::default(),
///         to: 1
///     }).await;
///     
///     let result = rx.recv().await;
///     assert_eq!(result, Some(LocalEvent::Terminate {
///         job_id: ResourceId::default(),
///         to: 1
///     }))
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum LocalEvent {
    Terminate {
        job_id: ResourceId,
        to: types::SinkId,
        event_time: i64,
    },
    KeyedDataStreamEvent(KeyedDataEvent),
}

unsafe impl Send for LocalEvent {}
unsafe impl Sync for LocalEvent {}

impl Serialize for LocalEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LocalEvent::Terminate {
                job_id,
                to,
                event_time,
            } => {
                let mut s = serializer.serialize_struct("Terminate", 3)?;
                s.serialize_field("job_id", job_id)?;
                s.serialize_field("to", to)?;
                s.serialize_field("event_time", event_time)?;
                s.end()
            }
            LocalEvent::KeyedDataStreamEvent(event) => {
                let mut s = serializer.serialize_struct("KeyedDataStreamEvent", 1)?;
                let ref mut buf = vec![];
                event.encode(buf);
                s.serialize_field("event", buf)?;
                s.end()
            }
        }
    }
}

#[derive(Debug)]
pub enum KafkaEventError {
    UnsupportedEvent,
    SerializeJsonFailed(String),
}

impl From<serde_json::Error> for KafkaEventError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeJsonFailed(format!(
            "serialize json failed, error occurs at line:{} and column: {}. error message: {}",
            err.line(),
            err.column(),
            err
        ))
    }
}

impl StreamEvent for LocalEvent {
    fn generate_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError> {
        match self {
            LocalEvent::Terminate { .. } => Err(KafkaEventError::UnsupportedEvent),
            LocalEvent::KeyedDataStreamEvent(e) => {
                let key = TypedValue::from_slice(
                    &e.key
                        .as_ref()
                        .map(|entry| entry.value.clone())
                        .unwrap_or_default(),
                )
                .to_json_value();
                let values = e
                    .data
                    .iter()
                    .map(|entry| TypedValue::from_slice(&entry.value).to_json_value());
                serde_json::to_vec(&key)
                    .and_then(|k| {
                        let mut messages = vec![];
                        let timestamp = chrono::DateTime::<chrono::Utc>::from(SystemTime::now());
                        for val in values {
                            let payload_result = serde_json::to_vec(&val);
                            if payload_result.is_err() {
                                return payload_result.map(|_| Default::default());
                            }

                            messages.push(KafkaMessage {
                                key: k.to_vec(),
                                payload: payload_result.unwrap(),
                                timestamp: Some(timestamp.timestamp_millis()),
                            })
                        }

                        Ok(messages)
                    })
                    .map_err(|err| err.into())
            }
        }
    }

    fn from_slice(slice: &[u8]) -> Self {
        todo!()
    }

    fn event_id(&self) -> i64 {
        match self {
            LocalEvent::Terminate {
                job_id: _,
                to: _,
                event_time,
            } => *event_time,
            LocalEvent::KeyedDataStreamEvent(event) => event.event_id,
        }
    }

    fn event_time(&self) -> i64 {
        match self {
            LocalEvent::Terminate {
                job_id: _,
                to: _,
                event_time,
            } => *event_time,
            LocalEvent::KeyedDataStreamEvent(event) => event.get_event_time(),
        }
    }
}

use std::time::SystemTime;

use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use serde::Deserialize;
use serde::Serialize;

use crate::kafka::KafkaMessage;
use crate::types::{self, TypedValue};

/// The trait that is a stream event
/// A [`StreamEvent`] can be serialized to be sent to external sink (e.g. Kafka, Database).
///
/// # Thread-safety
///
/// [`StreamEvent`] should implement [`Send`] and [`Sync`] which means:
/// - a [`StreamEvent`] can be shared between threads safely;
/// - you can use [tokio::sync::mpsc::Sender] and [tokio::sync::mpsc::Receiver] to transfer a [`StreamEvent`] across threads;
///
/// # Serialize
///
/// [`StreamEvent`] also implements [serde::Serialize]. You can serialize a [`StreamEvent`] if the serializer supports serde.
pub trait StreamEvent: Send + Sync + Serialize + Sized {
    /// Each stream event can be transformed to kafka message.
    fn to_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError>;
    /// Deserialize from slice.
    /// Deserializer will be choosed
    fn from_slice(slice: &[u8]) -> Result<Self, StreamEventDeserializeError>;
    /// stream event may has a id
    fn event_id(&self) -> i64;
    /// stream event may has a timestamp to indicate when it is generated
    fn event_time(&self) -> i64;
}

#[derive(Debug)]
pub enum StreamEventDeserializeError {
    RmpDecodeError(rmp_serde::decode::Error),
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq)]
pub enum LocalEvent {
    Terminate {
        job_id: ResourceId,
        to: types::SinkId,
        event_time: i64,
    },
    KeyedDataStreamEvent(KeyedDataEvent),
}

impl PartialOrd for LocalEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            LocalEvent::KeyedDataStreamEvent(e1) => match other {
                LocalEvent::KeyedDataStreamEvent(e2) => e1.event_id.partial_cmp(&e2.event_id),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Ord for LocalEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(order) => order,
            None => std::cmp::Ordering::Equal,
        }
    }
}

impl LocalEvent {
    pub fn set_to_operator_id(&mut self, to_operator_id: u32) {
        match self {
            LocalEvent::KeyedDataStreamEvent(e) => e.to_operator_id = to_operator_id,
            _ => {}
        }
    }
}

unsafe impl Send for LocalEvent {}
unsafe impl Sync for LocalEvent {}

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
    fn to_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError> {
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
                                key: bytes::Bytes::copy_from_slice(&k),
                                payload: bytes::Bytes::from(payload_result.unwrap()),
                                timestamp: Some(timestamp.timestamp_millis()),
                            })
                        }

                        Ok(messages)
                    })
                    .map_err(|err| err.into())
            }
        }
    }

    fn from_slice(slice: &[u8]) -> Result<Self, StreamEventDeserializeError> {
        rmp_serde::from_slice(slice).map_err(|err| StreamEventDeserializeError::RmpDecodeError(err))
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

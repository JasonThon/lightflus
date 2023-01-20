use std::time::SystemTime;

use proto::common::KeyedDataEvent;
use proto::common::ResourceId;

use crate::kafka::KafkaMessage;
use crate::types::{self, TypedValue};

pub trait KeyedEvent<K, V> {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc>;
    fn get_key(&self) -> K;
    fn get_value(&self) -> V;
}

/// [`LocalEvent`] can be only transmitted in a single thread. 
/// [`LocalEvent`] cannot be serialized to be transferred through pipe or network.
/// # Example
/// 
/// ```
/// use common::event::LocalEvent
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
///     assert!(result.is_ok());
///     
///     let event = result.unwrap();
///     assert_eq!(event, LocalEvent::Terminate {
///         job_id: ResourceId::default(),
///         to: 1
///     })
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum LocalEvent {
    Terminate {
        job_id: ResourceId,
        to: types::SinkId,
    },
    KeyedDataStreamEvent(KeyedDataEvent),
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

pub trait SinkableMessage {
    fn get_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum SinkableMessageImpl {
    LocalMessage(LocalEvent),
}

impl SinkableMessage for SinkableMessageImpl {
    fn get_kafka_message(&self) -> Result<Vec<KafkaMessage>, KafkaEventError> {
        match self {
            SinkableMessageImpl::LocalMessage(event) => match event {
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
                            let timestamp =
                                chrono::DateTime::<chrono::Utc>::from(SystemTime::now());
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
            },
        }
    }
}

unsafe impl Send for SinkableMessageImpl {}
unsafe impl Sync for SinkableMessageImpl {}

use std::{
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{ready, FutureExt, StreamExt};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message,
};

use crate::err::KafkaException;

pub fn run_consumer(
    brokers: &str,
    group_id: &str,
    topic: &str,
) -> Result<KafkaConsumer, rdkafka::error::KafkaError> {
    let group_id = if group_id.is_empty() {
        "lightflus"
    } else {
        group_id
    };

    let consumer_result: Result<StreamConsumer, rdkafka::error::KafkaError> = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "beginning")
        .create();
    consumer_result.and_then(|consumer| {
        consumer
            .subscribe(&[topic])
            .map(|_| KafkaConsumer::new(consumer))
    })
}

pub fn run_producer(
    brokers: &str,
    topic: &str,
    group: &str,
    partition: i32,
) -> Result<KafkaProducer, rdkafka::error::KafkaError> {
    ClientConfig::new()
        .set("group.id", group)
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "3000")
        .create()
        .and_then(|producer| {
            Ok(KafkaProducer {
                producer,
                topic: topic.to_string(),
                partition,
            })
        })
}

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
    partition: i32,
}

impl KafkaProducer {
    pub async fn send(&self, key: &[u8], payload: &[u8]) -> Result<(), KafkaException> {
        if payload.is_empty() {
            Ok(())
        } else {
            let record = FutureRecord::to(self.topic.as_str())
                .partition(self.partition)
                .payload(payload)
                .key(key);
            self.producer
                .send(record, Duration::from_secs(3))
                .await
                .map(|(partition, offset)| {
                    tracing::debug!(
                        "send message to partition {} with offset {}",
                        partition,
                        offset
                    )
                })
                .map_err(|err| KafkaException { err: err.0 })
        }
    }

    pub fn close(&mut self) {
        self.topic.clear();
        drop(self.partition);
    }
}

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

/// A wrapper of kafka message with key, payload and timestamp
/// Differently, key and payload are [bytes::Bytes] which may zero-copy when sharing kafka message between threads
#[derive(Clone, Debug)]
pub struct KafkaMessage {
    pub key: bytes::Bytes,
    pub payload: bytes::Bytes,
    pub timestamp: Option<i64>,
}

impl KafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        Self { consumer }
    }

    pub async fn fetch<M, F: FnMut(KafkaMessage) -> M>(&self, mut processor: F) -> Option<M> {
        self.consumer
            .stream()
            .next()
            .await
            .and_then(|msg| match msg {
                Ok(msg) => {
                    let msg = msg.detach();
                    msg.payload().map(|payload| {
                        let key = msg
                            .key()
                            .map(|key| bytes::Bytes::copy_from_slice(key))
                            .unwrap_or_default();
                        processor(KafkaMessage {
                            key,
                            payload: bytes::Bytes::copy_from_slice(payload),
                            timestamp: msg.timestamp().to_millis(),
                        })
                    })
                }
                Err(err) => {
                    tracing::error!("fail to fetch data from kafka: {}", err);
                    None
                }
            })
    }

    pub fn blocking_fetch<M, F: Fn(KafkaMessage) -> M>(&self, processor: F) -> Option<M> {
        futures_executor::block_on_stream(self.consumer.stream())
            .next()
            .and_then(|result| match result {
                Ok(msg) => {
                    let msg = msg.detach();
                    msg.payload().map(|payload| {
                        let key = msg
                            .key()
                            .map(|key| bytes::Bytes::copy_from_slice(key))
                            .unwrap_or_default();
                        processor(KafkaMessage {
                            key,
                            payload: bytes::Bytes::copy_from_slice(payload),
                            timestamp: msg.timestamp().to_millis(),
                        })
                    })
                }
                Err(err) => {
                    tracing::error!("fail to fetch data from kafka: {}", err);
                    None
                }
            })
    }

    pub fn unsubscribe(&self) {
        self.consumer.unsubscribe();
    }
}

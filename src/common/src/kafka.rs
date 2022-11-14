use std::time::Duration;

use futures_util::StreamExt;
use proto::common::stream::KafkaDesc_KafkaOptions;
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
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "beginning")
        .create()
        .expect("Consumer creation failed");
    consumer
        .subscribe(&[topic])
        .map(|_| KafkaConsumer::new(consumer))
}

pub fn run_producer(brokers: &str, topic: &str, opts: &KafkaDesc_KafkaOptions) -> KafkaProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("group.id", opts.get_group())
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "3000")
        .create()
        .expect("Producer creation error");

    let partition = if opts.has_partition() {
        opts.get_partition() as i32
    } else {
        0
    };

    KafkaProducer {
        producer,
        topic: topic.to_string(),
        partition,
    }
}

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
    partition: i32,
}

impl KafkaProducer {
    pub fn send(&self, key: &[u8], payload: &[u8]) -> Result<(), KafkaException> {
        if payload.is_empty() {
            Ok(())
        } else {
            futures_executor::block_on(async {
                let record = FutureRecord::to(self.topic.as_str())
                    .partition(self.partition)
                    .payload(payload)
                    .key(key);
                self.producer
                    .send(record, Duration::from_secs(3))
                    .await
                    .map(|(partition, offset)| {
                        log::debug!(
                            "send message to partition {} with offset {}",
                            partition,
                            offset
                        )
                    })
                    .map_err(|err| KafkaException { err: err.0 })
            })
        }
    }
}

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

#[derive(Clone)]
pub struct KafkaMessage {
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
}

impl KafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        Self { consumer }
    }

    pub fn fetch<M, F: FnMut(KafkaMessage) -> M>(&self, mut processor: F) -> Option<M> {
        futures_executor::block_on(self.consumer.stream().next()).and_then(|msg| match msg {
            Ok(msg) => {
                let msg = msg.detach();
                msg.payload().map(|payload| {
                    let key = msg.key().map(|key| key.to_vec()).unwrap_or_default();
                    processor(KafkaMessage {
                        key,
                        payload: payload.to_vec(),
                    })
                })
            }
            Err(err) => {
                log::error!("fail to fetch data from kafka: {}", err);
                None
            }
        })
    }
}

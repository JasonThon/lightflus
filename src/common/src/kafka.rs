use std::time::Duration;

use proto::common::stream::KafkaDesc_KafkaOptions;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

pub fn run_consumer(
    brokers: &str,
    group_id: &str,
    topic: &str,
) -> Result<StreamConsumer, rdkafka::error::KafkaError> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()
        .expect("Consumer creation failed");
    consumer.subscribe(&[topic]).map(|_| consumer)
}

pub fn run_producer(brokers: &str, topic: &str, opts: &KafkaDesc_KafkaOptions) -> KafkaProducer {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
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
    pub fn send(&self, key: &[u8], payload: &[u8]) -> Result<(), KafkaError> {
        if payload.is_empty() {
            Ok(())
        } else {
            futures::executor::block_on(async {
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
                    .map_err(|err| err.0)
            })
        }
    }
}

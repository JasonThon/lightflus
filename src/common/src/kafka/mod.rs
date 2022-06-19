use crate::lists;

pub fn new_kafka_consumer(brokers: &Vec<String>, topics: Vec<&str>, group: &str) -> rdkafka::consumer::StreamConsumer {
    use rdkafka::consumer::Consumer;
    use rdkafka::config::FromClientConfig;
    use rdkafka::consumer::stream_consumer;

    let mut config = rdkafka::ClientConfig::new();
    let consumer: stream_consumer::StreamConsumer = config.set("bootstrap.servers", lists::to_string(brokers))
        .set("group.id", group)
        .set("allow.auto.create.topics", "true")
        .set_log_level(rdkafka::config::RDKafkaLogLevel::Error)
        .create()
        .expect("invalid kafka config");

    consumer
        .subscribe(topics.as_slice())
        .expect("subscribe topic failed");

    consumer
}
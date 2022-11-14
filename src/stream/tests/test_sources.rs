use std::collections::BTreeMap;

use common::{
    event::LocalEvent,
    kafka::{run_consumer, run_producer, KafkaMessage},
    types::TypedValue,
    utils::get_env,
};
use proto::{
    common::{
        common::{DataTypeEnum, ResourceId},
        event::{Entry, KeyedDataEvent},
        stream::{KafkaDesc, KafkaDesc_KafkaOptions},
    },
    worker::worker::DispatchDataEventStatusEnum,
};
use protobuf::RepeatedField;
use stream::actor::{Kafka, Sink, SinkableMessageImpl, Source};

#[tokio::test]
async fn test_kafka_source() {
    let mut kafka_desc = KafkaDesc::default();
    let mut kafka_opts = KafkaDesc_KafkaOptions::default();
    let kafka_host = get_env("KAFKA_HOST").unwrap_or("localhost".to_string());

    {
        kafka_opts.set_group("ci_group".to_string());
    }

    {
        kafka_desc.set_brokers(RepeatedField::from_slice(&[format!("{kafka_host}:9092")]));
        kafka_desc.set_data_type(DataTypeEnum::DATA_TYPE_ENUM_STRING);
        kafka_desc.set_opts(kafka_opts.clone());
        kafka_desc.set_topic("ci".to_string());
    }

    let kafka_source = Kafka::with_source_config(&ResourceId::default(), 0, &kafka_desc);

    let producer = run_producer(format!("{kafka_host}:9092").as_str(), "ci", &kafka_opts);

    let result = producer.send("key".as_bytes(), "value".as_bytes());
    assert!(result.is_ok());
    let msg = kafka_source.fetch_msg();
    assert!(msg.is_some());

    let msg = msg.unwrap();
    match msg {
        SinkableMessageImpl::LocalMessage(event) => match event {
            LocalEvent::KeyedDataStreamEvent(e) => {
                assert_eq!(e.get_data().len(), 1);
                assert_eq!(
                    e.get_data()[0].get_data_type(),
                    DataTypeEnum::DATA_TYPE_ENUM_STRING
                );
                let value = TypedValue::from_slice(e.get_data()[0].get_value());
                assert_eq!(value.get_type(), DataTypeEnum::DATA_TYPE_ENUM_STRING);
                match value {
                    TypedValue::String(v) => assert_eq!(v.as_str(), "value"),
                    _ => panic!("unexpected type"),
                }
                assert!(e.has_event_time());
            }
            _ => panic!("unexpected event"),
        },
    }
}
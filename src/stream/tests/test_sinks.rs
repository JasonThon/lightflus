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
async fn test_kafka_sink() {
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

    let kafka_sink = Kafka::with_sink_config(&ResourceId::default(), 0, &kafka_desc);
    let consumer = run_consumer(format!("{kafka_host}:9092").as_str(), "ci_group", "ci");
    assert!(consumer.is_ok());

    let consumer = consumer.unwrap();
    let mut event = KeyedDataEvent::default();

    let mut entry = Entry::default();
    entry.set_data_type(DataTypeEnum::DATA_TYPE_ENUM_OBJECT);

    let mut val = BTreeMap::default();
    val.insert("key_1".to_string(), TypedValue::String("val_1".to_string()));
    val.insert("key_2".to_string(), TypedValue::Number(1.0));

    let value = TypedValue::Object(val);
    entry.set_value(value.get_data());
    entry.set_data_type(value.get_type());

    event.set_job_id(ResourceId::default());
    event.set_data(RepeatedField::from_slice(&[entry.clone(), entry]));

    let result = kafka_sink.sink(SinkableMessageImpl::LocalMessage(
        LocalEvent::KeyedDataStreamEvent(event),
    ));
    if result.is_err() {
        panic!("{:?}", result.unwrap_err());
    }

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status, DispatchDataEventStatusEnum::DONE);

    fn processor(message: KafkaMessage) {
        let key = serde_json::from_slice::<serde_json::Value>(&message.key);
        assert!(key.is_ok());
        let value = serde_json::from_slice::<serde_json::Value>(&message.payload);
        assert!(value.is_ok());

        let key = key.unwrap();
        let value = value.unwrap();

        assert!(key.is_null());
        assert!(value.is_object());
    }

    let opt = consumer.fetch(processor);
    assert!(opt.is_some());

    let opt = consumer.fetch(processor);
    assert!(opt.is_some());
}

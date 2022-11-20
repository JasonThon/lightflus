use common::{
    event::{LocalEvent, SinkableMessageImpl},
    kafka::run_producer,
    types::TypedValue,
    utils::get_env,
};

use proto::common::{kafka_desc::KafkaOptions, DataTypeEnum, KafkaDesc, ResourceId};
use protobuf::RepeatedField;
use stream::actor::{Kafka, Source};

#[tokio::test]
async fn test_kafka_source() {
    let kafka_desc = KafkaDesc {
        brokers: vec![get_env("KAFKA_HOST").unwrap_or("localhost".to_string())],
        topic: "ci".to_string(),
        opts: Some(KafkaOptions {
            group: Some("ci_group".to_string()),
            partition: None,
        }),
        data_type: DataTypeEnum::String,
    };

    let kafka_source = Kafka::with_source_config(
        &ResourceId {
            resource_id: b"resource_id",
            namespace_id: b"default",
        },
        0,
        &kafka_desc,
    );

    let producer = run_producer(format!("{kafka_host}:9092").as_str(), "ci", &kafka_opts);
    assert!(producer.is_ok());
    let producer = producer.unwrap();

    let result = producer.send("key".as_bytes(), "value".as_bytes());
    assert!(result.is_ok());
    let msg = kafka_source.fetch_msg();
    assert!(msg.is_some());

    let msg = msg.unwrap();
    match msg {
        SinkableMessageImpl::LocalMessage(event) => match event {
            LocalEvent::KeyedDataStreamEvent(e) => {
                assert_eq!(e.data.len(), 1);
                assert_eq!(e.data[0].data_type(), DataTypeEnum::String);
                let value = TypedValue::from_slice(e.get_data()[0].get_value());
                assert_eq!(value.get_type(), DataTypeEnum::String);
                match value {
                    TypedValue::String(v) => assert_eq!(v.as_str(), "value"),
                    _ => panic!("unexpected type"),
                }
                assert!(e.event_time.is_some());
            }
            _ => panic!("unexpected event"),
        },
    }
}

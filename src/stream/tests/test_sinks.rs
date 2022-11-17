use std::collections::BTreeMap;

use bytes::Buf;
use common::{
    event::{LocalEvent, SinkableMessageImpl},
    kafka::{run_consumer, KafkaMessage},
    redis::RedisClient,
    types::TypedValue,
    utils::get_env,
};
use proto::{
    common::{
        common::{DataTypeEnum, ResourceId},
        event::{Entry, KeyedDataEvent},
        stream::{Func, KafkaDesc, KafkaDesc_KafkaOptions, RedisDesc, RedisDesc_ConnectionOpts},
    },
    worker::worker::DispatchDataEventStatusEnum,
};
use protobuf::RepeatedField;
use stream::actor::{Kafka, Redis, Sink, SinkImpl};

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

    let kafka_sink = SinkImpl::Kafka(Kafka::with_sink_config(
        &ResourceId::default(),
        1,
        &kafka_desc,
    ));

    assert_eq!(kafka_sink.sink_id(), 1);

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

// #[tokio::test]
// async fn test_redis_sink_success() {
//     let mut redis_desc = RedisDesc::default();
//     let mut key_extractor = Func::default();
//     key_extractor.set_function("function (a) {a.key}".to_string());
//     let mut value_extractor = Func::default();
//     value_extractor.set_function("function (a) {a.value}".to_string());

//     {
//         let mut opts = RedisDesc_ConnectionOpts::default();
//         opts.set_database(0);
//         opts.set_host(get_env("REDIS_HOST").unwrap_or("localhost".to_string()));
//         redis_desc.set_key_extractor(key_extractor);
//         redis_desc.set_value_extractor(value_extractor);
//         redis_desc.set_connection_opts(opts);
//     }

//     let redis_sink = SinkImpl::Redis(Redis::with_config(1, redis_desc.clone()));

//     assert_eq!(redis_sink.sink_id(), 1);

//     let mut event = KeyedDataEvent::default();
//     let mut entry = Entry::default();
//     entry.set_data_type(DataTypeEnum::DATA_TYPE_ENUM_OBJECT);

//     let mut val = BTreeMap::default();
//     val.insert("key".to_string(), TypedValue::String("word-1".to_string()));
//     val.insert("value".to_string(), TypedValue::BigInt(10));

//     let value = TypedValue::Object(val);
//     entry.set_value(value.get_data());
//     entry.set_data_type(value.get_type());

//     let mut entry_2 = Entry::default();
//     entry_2.set_data_type(DataTypeEnum::DATA_TYPE_ENUM_OBJECT);

//     let mut val = BTreeMap::default();
//     val.insert("key".to_string(), TypedValue::String("word-2".to_string()));
//     val.insert("value".to_string(), TypedValue::BigInt(100));

//     let value = TypedValue::Object(val);
//     entry_2.set_value(value.get_data());
//     entry_2.set_data_type(value.get_type());

//     event.set_job_id(ResourceId::default());
//     event.set_data(RepeatedField::from_slice(&[entry, entry_2]));

//     let result = redis_sink.sink(SinkableMessageImpl::LocalMessage(
//         LocalEvent::KeyedDataStreamEvent(event),
//     ));

//     assert!(result.is_ok());

//     let client = RedisClient::new(&redis_desc);
//     let conn_result = client.connect();
//     assert!(conn_result.is_ok());

//     let ref mut conn = conn_result.expect("");
//     let result = client.get(conn, &TypedValue::String("word-1".to_string()));
//     assert!(result.is_ok());
//     let value = result.expect("msg");

//     assert_eq!(value.as_slice().get_i64(), 10);

//     let result = client.get(conn, &TypedValue::String("word-2".to_string()));
//     assert!(result.is_ok());
//     let value = result.expect("msg");

//     assert_eq!(value.as_slice().get_i64(), 100)
// }

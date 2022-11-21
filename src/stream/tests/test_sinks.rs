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
        kafka_desc, redis_desc, DataTypeEnum, Entry, Func, KafkaDesc, KeyedDataEvent, RedisDesc,
        ResourceId,
    },
    worker::DispatchDataEventStatusEnum,
};
use stream::actor::{Kafka, Redis, Sink, SinkImpl};

struct SetupGuard {}

impl Drop for SetupGuard {
    fn drop(&mut self) {}
}

fn setup() -> SetupGuard {
    static MOD_TEST_START: std::sync::Once = std::sync::Once::new();
    MOD_TEST_START.call_once(|| {
        v8::V8::set_flags_from_string(
            "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
          );
              v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
              v8::V8::initialize();
    });

    SetupGuard {}
}

#[tokio::test]
async fn test_kafka_sink() {
    let kafka_host = get_env("KAFKA_HOST").unwrap_or("localhost".to_string());
    let kafka_sink = SinkImpl::Kafka(Kafka::with_sink_config(
        &ResourceId::default(),
        1,
        &KafkaDesc {
            brokers: vec![format!("{kafka_host}:9092")],
            topic: "ci".to_string(),
            opts: Some(kafka_desc::KafkaOptions {
                group: Some("ci_group".to_string()),
                partition: Some(0),
            }),
            data_type: DataTypeEnum::String as i32,
        },
    ));

    assert_eq!(kafka_sink.sink_id(), 1);

    let consumer = run_consumer(format!("{kafka_host}:9092").as_str(), "ci_group", "ci");
    assert!(consumer.is_ok());

    let consumer = consumer.unwrap();
    let event = KeyedDataEvent {
        job_id: Some(ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespaceId".to_string(),
        }),
        key: None,
        to_operator_id: 1,
        data: vec![
            Entry {
                data_type: DataTypeEnum::Object as i32,
                value: TypedValue::Object(BTreeMap::from_iter(
                    [
                        ("key_1".to_string(), TypedValue::String("val_1".to_string())),
                        ("key_2".to_string(), TypedValue::Number(1.0)),
                    ]
                    .iter()
                    .map(|entry| (entry.0.clone(), entry.1.clone())),
                ))
                .get_data(),
            },
            Entry {
                data_type: DataTypeEnum::Object as i32,
                value: TypedValue::Object(BTreeMap::from_iter(
                    [
                        ("key_1".to_string(), TypedValue::String("val_1".to_string())),
                        ("key_2".to_string(), TypedValue::Number(1.0)),
                    ]
                    .iter()
                    .map(|entry| (entry.0.clone(), entry.1.clone())),
                ))
                .get_data(),
            },
        ],
        event_time: None,
        process_time: None,
        from_operator_id: 0,
        window: None,
    };

    let result = kafka_sink
        .sink(SinkableMessageImpl::LocalMessage(
            LocalEvent::KeyedDataStreamEvent(event),
        ))
        .await;
    if result.is_err() {
        panic!("{:?}", result.unwrap_err());
    }

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status, DispatchDataEventStatusEnum::Done);

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

    let opt = consumer.fetch(processor).await;
    assert!(opt.is_some());

    let opt = consumer.fetch(processor).await;
    assert!(opt.is_some());
}

#[tokio::test]
async fn test_redis_sink_success() {
    let _setup_guard = setup();
    let ref desc = RedisDesc {
        connection_opts: Some(redis_desc::ConnectionOpts {
            host: get_env("REDIS_HOST").unwrap_or("localhost".to_string()),
            username: Default::default(),
            password: Default::default(),
            database: 0,
            tls: false,
        }),
        key_extractor: Some(Func {
            function: "function redis_extractor(a) {return a.key}".to_string(),
        }),
        value_extractor: Some(Func {
            function: "function redis_extractor(a) {return a.value}".to_string(),
        }),
    };

    let redis_sink = SinkImpl::Redis(Redis::with_config(1, desc));

    assert_eq!(redis_sink.sink_id(), 1);

    let event = KeyedDataEvent {
        job_id: Some(ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespaceId".to_string(),
        }),
        key: None,
        to_operator_id: 1,
        data: vec![
            Entry {
                data_type: DataTypeEnum::Object as i32,
                value: TypedValue::Object(BTreeMap::from_iter(
                    [
                        ("key".to_string(), TypedValue::String("word-1".to_string())),
                        ("value".to_string(), TypedValue::BigInt(10)),
                    ]
                    .iter()
                    .map(|entry| (entry.0.clone(), entry.1.clone())),
                ))
                .get_data(),
            },
            Entry {
                data_type: DataTypeEnum::Object as i32,
                value: TypedValue::Object(BTreeMap::from_iter(
                    [
                        ("key".to_string(), TypedValue::String("word-2".to_string())),
                        ("value".to_string(), TypedValue::BigInt(100)),
                    ]
                    .iter()
                    .map(|entry| (entry.0.clone(), entry.1.clone())),
                ))
                .get_data(),
            },
        ],
        event_time: None,
        process_time: None,
        from_operator_id: 0,
        window: None,
    };

    let result = redis_sink
        .sink(SinkableMessageImpl::LocalMessage(
            LocalEvent::KeyedDataStreamEvent(event),
        ))
        .await;

    assert!(result.is_ok());

    let client = RedisClient::new(&desc);
    let conn_result = client.connect();
    assert!(conn_result.is_ok());

    let ref mut conn = conn_result.expect("");
    let result = client.get(conn, &TypedValue::String("word-1".to_string()));
    assert!(result.is_ok());
    let value = result.expect("msg");

    assert_eq!(value.as_slice().get_i64(), 10);

    let result = client.get(conn, &TypedValue::String("word-2".to_string()));
    assert!(result.is_ok());
    let value = result.expect("msg");

    assert_eq!(value.as_slice().get_i64(), 100)
}

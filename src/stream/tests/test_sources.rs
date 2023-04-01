use std::{
    pin::Pin,
    task::{Context, Poll},
};

use common::{event::LocalEvent, kafka::run_producer, types::TypedValue, utils::get_env};

use futures_util::{ready, Future};
use proto::common::{kafka_desc::KafkaOptions, DataTypeEnum, KafkaDesc, ResourceId};
use stream::connector::{Kafka, Source};

#[tokio::test]
async fn test_kafka_source_async_fetch_msg() {
    let kafka_host = get_env("KAFKA_HOST").unwrap_or("localhost".to_string());
    let kafka_desc = KafkaDesc {
        brokers: vec![format!("{kafka_host}:9092")],
        topic: "ci".to_string(),
        opts: Some(KafkaOptions {
            group: Some("ci_group".to_string()),
            partition: None,
        }),
        data_type: DataTypeEnum::String as i32,
    };

    let mut kafka_source = Kafka::with_source_config(
        &ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "default".to_string(),
        },
        0,
        &kafka_desc,
    );

    let producer = run_producer(format!("{kafka_host}:9092").as_str(), "ci", "ci_group", 0);
    assert!(producer.is_ok());
    let producer = producer.unwrap();

    let result = producer.send("key".as_bytes(), "value".as_bytes()).await;
    assert!(result.is_ok());
    let msg = kafka_source.next().await;
    assert!(msg.is_some());

    let msg = msg.unwrap();
    match msg {
        LocalEvent::KeyedDataStreamEvent(e) => {
            assert_eq!(e.data.len(), 1);
            assert_eq!(e.data[0].data_type(), DataTypeEnum::String);
            let value = TypedValue::from_slice(&e.data[0].value);
            assert_eq!(value.get_type(), DataTypeEnum::String);
            match value {
                TypedValue::String(v) => assert_eq!(v.as_str(), "value"),
                _ => panic!("unexpected type"),
            }
        }
        _ => panic!("unexpected event"),
    }
}

#[tokio::test]
async fn test_kafka_source_poll_next() {
    let kafka_host = get_env("KAFKA_HOST").unwrap_or("localhost".to_string());
    let kafka_desc = KafkaDesc {
        brokers: vec![format!("{kafka_host}:9092")],
        topic: "ci".to_string(),
        opts: Some(KafkaOptions {
            group: Some("ci_group".to_string()),
            partition: None,
        }),
        data_type: DataTypeEnum::String as i32,
    };

    let kafka_source = Kafka::with_source_config(
        &ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "default".to_string(),
        },
        0,
        &kafka_desc,
    );

    let producer = run_producer(format!("{kafka_host}:9092").as_str(), "ci", "ci_group", 0);
    assert!(producer.is_ok());
    let producer = producer.unwrap();
    let result = producer.send("key".as_bytes(), "value".as_bytes()).await;
    assert!(result.is_ok());

    struct TestKafkaPoll {
        kafka: Kafka,
    }

    impl Future for TestKafkaPoll {
        type Output = Result<(), bool>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();

            loop {
                let opt = ready!(this.kafka.poll_next(cx));
                match opt {
                    Some(event) => match event {
                        LocalEvent::KeyedDataStreamEvent(e) => {
                            assert_eq!(e.data.len(), 1);
                            assert_eq!(e.data[0].data_type(), DataTypeEnum::String);
                            let value = TypedValue::from_slice(&e.data[0].value);
                            assert_eq!(value.get_type(), DataTypeEnum::String);
                            match value {
                                TypedValue::String(v) => assert_eq!(v.as_str(), "value"),
                                _ => panic!("unexpected type"),
                            }
                            break;
                        }
                        _ => panic!("unexpected event"),
                    },
                    None => continue,
                }
            }

            Poll::Ready(Ok(()))
        }
    }

    let test_kafka_poll = TestKafkaPoll {
        kafka: kafka_source,
    };

    let result = tokio::spawn(test_kafka_poll).await;
    assert!(result.is_ok())
}

use std::{rc::Rc, time::Duration};

use common::event::{LocalEvent, SinkableMessageImpl};
use proto::common::{DataTypeEnum, KafkaDesc, OperatorInfo, ResourceId};
use stream::actor::{ExecutorImpl, Kafka, LocalExecutor, SinkImpl, SourceImpl};

#[tokio::test]
async fn test_actor_close() {
    let job_id = ResourceId {
        resource_id: "resource_id".to_string(),
        namespace_id: "ns_id".to_string(),
    };
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    let innert_tx = tx.clone();
    let source = SourceImpl::Empty(0, tx, Rc::new(rx));
    let sinks = vec![SinkImpl::Kafka(Kafka::with_sink_config(
        &job_id,
        1,
        &KafkaDesc {
            brokers: vec!["localhost:9092".to_string()],
            topic: "topic".to_string(),
            opts: None,
            data_type: DataTypeEnum::Object as i32,
        },
    ))];
    let operator = OperatorInfo::default();
    let executor = LocalExecutor::with_source_and_sink(&job_id, 0, sinks, source, operator);
    let executor = ExecutorImpl::Local(executor);
    let handler = tokio::spawn(async { executor.run() });
    let result = innert_tx
        .send(SinkableMessageImpl::LocalMessage(LocalEvent::Terminate {
            job_id,
            to: 0,
        }))
        .await;
    assert!(result.is_ok());
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(handler.is_finished())
}

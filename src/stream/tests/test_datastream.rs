use std::ops::Add;
use std::time;

use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

use stream::{dataflow, pipeline::Context, window::KeyedWindow, window, trigger, pipeline};
use common::{types, event};

type MockDataStream = dataflow::DataStream<MockEvent, String, MockSink, MockPipeline, String, String, String>;

struct MockEvent {
    key: String,
    value: String,
    event_time: time::SystemTime,
}

impl event::Event<String, String> for MockEvent {
    fn event_time(&self) -> DateTime<Utc> {
        DateTime::from(self.event_time)
    }

    fn get_key(&self) -> String {
        self.key.to_string()
    }

    fn get_value(&self) -> String {
        self.value.to_string()
    }
}

struct MockPipeline {}

impl pipeline::Pipeline<String, String, String, String> for MockPipeline {
    type Context = ();

    fn apply(&self, input: &KeyedWindow<String, String>, _ctx: &Context<String, String>) -> pipeline::Result<String> {
        let mut result = String::new();
        for value in &input.values {
            result = result + ";" + value.as_str();
        }

        Ok(result)
    }

    fn create_context(&self) -> Context<String, String> {
        todo!()
    }
}

struct MockSink {
    tx: mpsc::UnboundedSender<String>,
}

impl dataflow::Sink<String> for MockSink {
    fn sink(&self, output: String) {
        match self.tx.send(output) {
            Ok(_) => log::info!("success sink"),
            Err(_) => log::error!("failed sink")
        }
    }
}

unsafe impl Send for MockSink {}

unsafe impl Sync for MockSink {}

fn new_datastream(sink_tx: mpsc::UnboundedSender<String>) -> (MockDataStream, dataflow::StreamPipeSender<Vec<MockEvent>>, mpsc::Sender<dataflow::Close>) {
    let (sender, recv) = dataflow::stream_pipe::<Vec<MockEvent>>();
    let (close_tx, close_rx) = mpsc::channel(1);
    let pipeline = MockPipeline {};
    let sink = MockSink {
        tx: sink_tx
    };
    let stream = dataflow::DataStream::new(
        window::WindowType::Fixed { size: time::Duration::from_secs(1) },
        trigger::TriggerType::Watermark {
            firetime: time::Duration::from_millis(100)
        },
        recv,
        close_rx,
        pipeline,
        sink,
    );

    (stream, sender, close_tx)
}


#[tokio::test]
async fn test_datastream_disconnect() {
    let (tx, _) = mpsc::unbounded_channel();
    let (stream, sender, close_tx) = new_datastream(tx);
    tokio::spawn(stream.start());

    let first_send = close_tx.send(dataflow::Close).await;
    assert!(first_send.is_ok());

    let second_send = close_tx.send(dataflow::Close).await;
    assert!(second_send.is_err());

    let result = sender.send(vec![]);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_datastream_pipeline_can_be_executed() {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let (stream, sender, close_tx) = new_datastream(tx);
    tokio::spawn(stream.start());
    let start_time = time::SystemTime::now();
    let events = vec![
        MockEvent {
            key: "key".to_string(),
            value: "value-1".to_string(),
            event_time: start_time,
        },
        MockEvent {
            key: "key".to_string(),
            value: "value-2".to_string(),
            event_time: start_time.add(time::Duration::from_millis(50)),
        },
        MockEvent {
            key: "key".to_string(),
            value: "value-3".to_string(),
            event_time: start_time.add(time::Duration::from_millis(100)),
        },
    ];
    let result = sender.send(events);
    assert!(result.is_ok());
    tokio::time::sleep(time::Duration::from_millis(1000)).await;
    let _ = close_tx.send(dataflow::Close).await;
    while let Some(result) = rx.recv().await {
        assert_eq!(result, ";value-1;value-2;value-3");
    }
}
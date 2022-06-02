use std::ops::Add;
use std::time;

use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

use dataflow::{event, stream as datastream, types};
use dataflow::stream::pipeline::Context;
use dataflow::stream::window::KeyedWindow;

type MockDataStream = datastream::DataStream<MockEvent, String, MockSink, MockPipeline, String, String, String>;

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

impl datastream::pipeline::Pipeline<String, String, String, String> for MockPipeline {
    type Context = ();

    fn apply(&self, input: &KeyedWindow<String, String>, _ctx: &Context<String, String>) -> datastream::pipeline::Result<String> {
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

impl datastream::Sink<String> for MockSink {
    fn sink(&self, output: String) {
        match self.tx.send(output) {
            Ok(_) => log::info!("success sink"),
            Err(_) => log::error!("failed sink")
        }
    }
}

unsafe impl Send for MockSink {}

unsafe impl Sync for MockSink {}

fn new_datastream(sink_tx: mpsc::UnboundedSender<String>) -> (MockDataStream, datastream::StreamPipeSender<Vec<MockEvent>>, mpsc::Sender<datastream::Close>) {
    let (sender, recv) = datastream::stream_pipe::<Vec<MockEvent>>();
    let (close_tx, close_rx) = mpsc::channel(1);
    let pipeline = MockPipeline {};
    let sink = MockSink {
        tx: sink_tx
    };
    let stream = datastream::DataStream::new(
        datastream::window::WindowType::Fixed { size: time::Duration::from_secs(1) },
        datastream::trigger::TriggerType::Watermark {
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

    let first_send = close_tx.send(datastream::Close).await;
    assert!(first_send.is_ok());

    let second_send = close_tx.send(datastream::Close).await;
    assert!(second_send.is_err());

    let result = sender.send(vec![]);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_datastream_pipeline_can_be_executed() {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let (stream, sender, close_tx) = new_datastream(tx);
    tokio::spawn(stream.start());
    let start_time = std::time::SystemTime::now();
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
    let _ = close_tx.send(datastream::Close).await;
    while let Some(result) = rx.recv().await {
        assert_eq!(result, ";value-1;value-2;value-3");
    }
}
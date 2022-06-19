use tokio::sync::mpsc;
use common::{event, types};

#[tokio::test]
async fn test_disconnect() {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let (mut disconnect_tx, disconnect_rx) = mpsc::channel(1);

    let ref desc = types::SourceDesc::Tableflow {
        host: "localhost".to_string(),
        port: 0,
        event_time: None,
    };

    let connector = dataflow::conn::Connector::new(desc, event_rx, disconnect_rx);
    let handle = tokio::spawn(connector.start());

    let result = disconnect_tx.send(event::Disconnect).await;
    assert!(result.is_ok());

    let r = handle.await;
    assert!(r.is_ok());

    let redisconnect = disconnect_tx.send(event::Disconnect).await;
    assert!(redisconnect.is_err());
    assert!(event_tx.send(vec![]).is_err());
}
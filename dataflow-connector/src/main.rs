use tokio::sync::mpsc;

use dataflow;
use dataflow::event;

mod api;

#[tokio::main]
async fn main() {
    log::set_max_level(log::LevelFilter::Info);
    let file_result = std::fs::File::open("dataflow-connector/etc/conn.json");
    if file_result.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", file_result.unwrap_err()))
    }
    let file = file_result.unwrap();
    let env_setup = common::sysenv::serde_env::from_reader(file);
    if env_setup.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", env_setup.unwrap_err()))
    }
    let value = env_setup.unwrap();

    let reader = serde_json::from_str::<dataflow::conn::ConnectionConfig>(value.as_str());
    if reader.is_err() {
        panic!("{}", format!("fail to parser config file: {:?}", reader.unwrap_err()))
    }

    let mut senders = vec![];
    let config = reader.unwrap();
    let mut disconnect_signals = vec![];
    let rt = tokio::runtime::Runtime::new().expect("thread pool allocate failed");

    common::lists::for_each(&config.sources, |source| {
        let (tx, rx) = mpsc::unbounded_channel();
        let (disconnect_tx, disconnect_rx) = mpsc::channel(1);

        senders.push(tx);
        disconnect_signals.push(disconnect_tx);
        rt.spawn(dataflow::conn::Connector::new(source, rx, disconnect_rx).start());
    });

    let _ = tokio::signal::ctrl_c().await;

    // close connector gracefully
    common::lists::for_each_mut(&mut disconnect_signals, |signal| {
        let _ = signal.try_send(event::Disconnect);
    });
}

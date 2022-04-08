use std::{collections, sync};

use tokio::sync::mpsc;

use dataflow::{cluster, coord, event};
use dataflow::err::CommonException;
use dataflow_api::dataflow_coordinator_grpc;

const DATAFLOW_DB: &str = "dataflow";

mod api;

#[tokio::main]
async fn main() {
    log::set_max_level(log::LevelFilter::Info);
    env_logger::init();
    let file_result = std::fs::File::open("etc/coord.json");
    if file_result.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", file_result.unwrap_err()))
    }
    let file = file_result.unwrap();
    let env_setup = common::sysenv::serde_env::from_reader(file);
    if env_setup.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", env_setup.unwrap_err()))
    }
    let value = env_setup.unwrap();

    let reader = serde_json::from_str::<dataflow::coord::CoordinatorConfig>(value.as_str());
    if reader.is_err() {
        panic!("{}", format!("fail to parser config file: {:?}", reader.unwrap_err()))
    }

    let config = reader.unwrap();
    let result = config.mongo.to_client();
    if result.is_err() {
        panic!("{}", format!("fail to connect mongo: {:?}", result.unwrap_err()))
    }


    let mut senders = vec![];
    let mut disconnect_signals = vec![];
    let rt = tokio::runtime::Runtime::new().expect("thread pool allocate failed");

    common::lists::for_each(&config.sources, |source| {
        let (tx, rx) = mpsc::unbounded_channel();
        let (disconnect_tx, disconnect_rx) = mpsc::channel(1);

        senders.push(tx);
        disconnect_signals.push(disconnect_tx);
        rt.spawn(dataflow::conn::Connector::new(source, rx, disconnect_rx).start());
    });

    let client = result.unwrap();
    let coordinator = coord::Coordinator::new(
        coord::JobRepo::Mongo(
            client.database(DATAFLOW_DB)
                .collection(coord::COORD_JOB_GRAPH_COLLECTION)
        ),
        senders,
    );

    let mut clusters = cluster::Cluster::new(&config.cluster);
    clusters.probe_state();

    let init_result = coordinator.init();
    match init_result {
        Err(err) => panic!("initialize failed: {:?}", err),
        Ok(models) => {
            rt.spawn(async move {
                let mut undispatched_queue = collections::VecDeque::new();

                for model in &models {
                    match model.dispatch() {
                        Err(err) => {
                            log::error!("dispatch model {:?} failed: {:?}", model, err);
                            undispatched_queue.push_back(model);
                        }
                        _ => {}
                    }
                }

                while !undispatched_queue.is_empty() {
                    let model = undispatched_queue.pop_front().unwrap();
                    match model.dispatch() {
                        Err(err) => {
                            log::error!("dispatch model {:?} failed: {:?}", model, err);
                            undispatched_queue.push_back(model);
                        }
                        _ => {}
                    }
                }
            });
        }
    }

    let server = api::CoordinatorApiImpl::new(coordinator, clusters);
    let service = dataflow_coordinator_grpc::create_coordinator_api(server);
    let mut grpc_server = grpcio::ServerBuilder::new(
        sync::Arc::new(grpcio::Environment::new(10)))
        .register_service(service)
        .bind("0.0.0.0", config.port as u16)
        .build()
        .expect("grpc server create failed");
    grpc_server.start();
    println!("service start at port: {}", &config.port);

    let _ = tokio::signal::ctrl_c().await;

    // close connector gracefully
    common::lists::for_each_mut(&mut disconnect_signals, |signal| {
        let _ = signal.try_send(event::Disconnect);
    });

    rt.shutdown_background();

    let _ = grpc_server.shutdown().await;
}
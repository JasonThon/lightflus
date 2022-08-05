use std::{collections, sync};

use tokio::sync::mpsc;

use common::{event, err::CommonException};
use proto::coordinator::coordinator_grpc;

const DATAFLOW_DB: &str = "dataflow";

mod api;
pub mod coord;
pub mod cluster;

#[tokio::main]
async fn main() {
    log::set_max_level(log::LevelFilter::Info);
    env_logger::init();
    let file_result = std::fs::File::open("src/coordinator/etc/coord.json");
    if file_result.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", file_result.unwrap_err()))
    }
    let file = file_result.unwrap();
    let env_setup = common::sysenv::serde_env::from_reader(file);
    if env_setup.is_err() {
        panic!("{}", format!("fail to read config file: {:?}", env_setup.unwrap_err()))
    }
    let value = env_setup.unwrap();

    let reader = serde_json::from_str::<coord::CoordinatorConfig>(value.as_str());
    if reader.is_err() {
        panic!("{}", format!("fail to parser config file: {:?}", reader.unwrap_err()))
    }

    let config = reader.unwrap();
    let result = config.mongo.to_client();
    if result.is_err() {
        panic!("{}", format!("fail to connect mongo: {:?}", result.unwrap_err()))
    }

    let rt = tokio::runtime::Runtime::new().expect("thread pool allocate failed");

    let client = result.unwrap();
    let coordinator = coord::Coordinator::new(
        coord::JobStorage::Mongo(
            client.database(DATAFLOW_DB)
                .collection(coord::COORD_JOB_GRAPH_COLLECTION)
        ),
        config.conn_proxy,
    );

    let mut clusters = cluster::Cluster::new(&config.cluster);
    clusters.probe_state();

    let server = api::CoordinatorApiImpl::new(coordinator, clusters);
    let service = coordinator_grpc::create_coordinator_api(server);
    let mut grpc_server = grpcio::ServerBuilder::new(
        sync::Arc::new(grpcio::Environment::new(10)))
        .register_service(service)
        .bind("0.0.0.0", config.port as u16)
        .build()
        .expect("grpc server create failed");
    grpc_server.start();
    println!("service start at port: {}", &config.port);

    let _ = tokio::signal::ctrl_c().await;

    rt.shutdown_background();

    let _ = grpc_server.shutdown().await;
}
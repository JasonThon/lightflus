use std::{fs, sync};

use common::{net::cluster::Cluster, utils};
use proto::coordinator::coordinator_grpc;

use crate::coord::Coordinator;

mod api;
pub mod coord;

#[tokio::main]
async fn main() {
    log::set_max_level(log::LevelFilter::Info);
    let config_file_path = utils::Args::default().arg("c").map(|arg| arg.value.clone());

    let file_result =
        fs::File::open(config_file_path.unwrap_or("src/coordinator/etc/coord.json".to_string()));
    if file_result.is_err() {
        panic!(
            "{}",
            format!("fail to read config file: {:?}", file_result.unwrap_err())
        )
    }
    let file = file_result.unwrap();
    let env_setup = common::utils::from_reader(file);
    if env_setup.is_err() {
        panic!(
            "{}",
            format!("fail to read config file: {:?}", env_setup.unwrap_err())
        )
    }
    let value = env_setup.unwrap();

    let reader = serde_json::from_str::<coord::CoordinatorConfig>(value.as_str());
    if reader.is_err() {
        panic!(
            "{}",
            format!("fail to parser config file: {:?}", reader.unwrap_err())
        )
    }

    let config = reader.unwrap();

    let rt = tokio::runtime::Runtime::new().expect("thread pool allocate failed");

    let coordinator = Coordinator::new(config.storage.to_dataflow_storage(), &config.cluster);

    let mut clusters = Cluster::new(&config.cluster);
    clusters.probe_state();

    let server = api::CoordinatorApiImpl::new(coordinator, clusters);
    let service = coordinator_grpc::create_coordinator_api(server);
    let mut grpc_server = grpcio::ServerBuilder::new(sync::Arc::new(grpcio::Environment::new(10)))
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

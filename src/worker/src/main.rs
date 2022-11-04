use common::utils;
use proto::worker::worker_grpc;
use std::fs;
use std::sync;
use stream::initialize_v8;

mod api;
pub mod manager;
pub mod worker;

#[tokio::main]
async fn main() {
    initialize_v8();
    let config_file_path = utils::Args::default().arg("c").map(|arg| arg.value.clone());

    let result =
        fs::File::open(config_file_path.unwrap_or("src/worker/etc/worker.json".to_string()));
    if result.is_err() {
        panic!(
            "{}",
            format!("config file open failed: {:?}", result.unwrap_err())
        )
    }
    env_logger::init();

    let file = result.unwrap();

    let env_setup = common::utils::from_reader(file);
    if env_setup.is_err() {
        panic!(
            "{}",
            format!("config file read failed: {:?}", env_setup.unwrap_err())
        )
    }

    let value = env_setup.unwrap();
    let reader = serde_json::from_str::<worker::TaskWorkerConfig>(value.as_str());

    if reader.is_err() {
        panic!(
            "{}",
            format!("config file parse failed: {:?}", reader.unwrap_err())
        )
    }

    let ref mut config = reader.unwrap();
    let task_worker = worker::new_worker();

    let server = api::TaskWorkerApiImpl::new(task_worker);
    let service = worker_grpc::create_task_worker_api(server);

    let grpc_server = grpcio::ServerBuilder::new(sync::Arc::new(grpcio::Environment::new(10)))
        .register_service(service)
        .bind("127.0.0.1", config.port as u16)
        .build();

    if grpc_server.is_err() {
        panic!("{:?}", grpc_server.unwrap_err())
    }

    let mut unwrap_server = grpc_server.unwrap();
    unwrap_server.start();
    log::info!("start service at port {}", &config.port);

    let _ = tokio::signal::ctrl_c().await;
    let _ = unwrap_server.shutdown().await;
}

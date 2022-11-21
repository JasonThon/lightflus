use common::utils;
use proto::worker::task_worker_api_server::TaskWorkerApiServer;
use std::fs;
use stream::initialize_v8;
use tonic::transport::Server;

mod api;
pub mod manager;
pub mod worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let server = TaskWorkerApiServer::new(api::TaskWorkerApiImpl::new(task_worker));
    let addr = format!("0.0.0.0:{}", config.port).parse()?;
    Server::builder().add_service(server).serve(addr).await?;

    let _ = tokio::signal::ctrl_c().await;
    Ok(())
}

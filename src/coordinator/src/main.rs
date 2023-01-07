use std::{fs, time::Duration};

use api::CoordinatorApiImpl;
use common::utils;
use proto::coordinator::coordinator_api_server::CoordinatorApiServer;
use tonic::transport::Server;

use crate::coord::Coordinator;

mod api;
pub mod coord;
mod managers;
mod scheduler;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
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

    let mut coordinator = Coordinator::new(&config);
    coordinator.probe_state().await;
    let server = CoordinatorApiImpl::new(coordinator);

    let addr = format!("0.0.0.0:{}", config.port).parse()?;
    tracing::info!("service will start at {}", config.port);

    Server::builder()
        .timeout(Duration::from_secs(3))
        .concurrency_limit_per_connection(5)
        .add_service(CoordinatorApiServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}

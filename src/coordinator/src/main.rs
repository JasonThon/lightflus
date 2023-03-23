use std::{fs, time::Duration};

use api::CoordinatorApiImpl;
use common::utils;
use lightflus_core::coordinator::coord;
use proto::coordinator::coordinator_api_server::CoordinatorApiServer;
use tonic::transport::Server;


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

    let reader = serde_json::from_str::<coord::CoordinatorBuilder>(value.as_str());
    if reader.is_err() {
        panic!(
            "{}",
            format!("fail to parser config file: {:?}", reader.unwrap_err())
        )
    }

    let mut builder = reader.unwrap();
    replace_builder_args_by_env(&mut builder);

    let coordinator = builder.build();

    let addr = format!("0.0.0.0:{}", builder.port).parse()?;
    tracing::info!("service will start at {}", builder.port);

    Server::builder()
        .timeout(Duration::from_secs(3))
        .concurrency_limit_per_connection(5)
        .add_service(coordinator)
        .serve(addr)
        .await?;

    Ok(())
}

fn replace_builder_args_by_env(builder: &mut coord::CoordinatorBuilder) {}

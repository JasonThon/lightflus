use common::utils::{self, get_env};
use proto::taskmanager::task_manager_api_server::TaskManagerApiServer;
use std::fs;
use stream::initialize_v8;
use tonic::transport::Server;

mod taskmanager;
pub mod taskworker;

const DEFAULT_WORKER_THREADS_NUM: usize = 100;

fn main() {
    let config_file_path = utils::Args::default().arg("c").map(|arg| arg.value.clone());

    let result =
        fs::File::open(config_file_path.unwrap_or("src/taskmanager/etc/taskmanager.json".to_string()));
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
    let reader = serde_json::from_str::<taskmanager::TaskManagerBuilder>(value.as_str());

    if reader.is_err() {
        panic!(
            "{}",
            format!("config file parse failed: {:?}", reader.unwrap_err())
        )
    }

    let worker_threads = get_env("WORKER_THREADS")
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WORKER_THREADS_NUM);

    let ref mut builder = reader.unwrap();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            tracing_subscriber::fmt::init();

            let server = TaskManagerApiServer::new(builder.build());
            let addr = format!("0.0.0.0:{}", builder.port).parse().unwrap();

            tracing::info!("service will start at {}", builder.port);

            initialize_v8();
            let _ = Server::builder().add_service(server).serve(addr).await;
        });
}

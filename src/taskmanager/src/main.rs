use common::utils::get_env;
use lightflus_core::taskmanager::rpc::load_builder;

use stream::initialize_v8;
use tonic::transport::Server;

const DEFAULT_WORKER_THREADS_NUM: usize = 100;

fn main() {
    tracing_subscriber::fmt::init();
    let worker_threads = get_env("WORKER_THREADS")
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WORKER_THREADS_NUM);

    let ref mut builder = load_builder();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let server = builder.build();
            let addr = format!("0.0.0.0:{}", builder.port).parse().unwrap();

            tracing::info!("service will start at {}", builder.port);

            initialize_v8();
            let _ = Server::builder().add_service(server).serve(addr).await;
        });
}

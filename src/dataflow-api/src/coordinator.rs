use crate::dataflow_coordinator_grpc;
use std::sync;

#[derive(Clone, serde::Deserialize)]
pub struct CoordinatorApiConfig {
    pub host: String,
    pub port: u16,
}

pub fn new_coordinator_client(config: CoordinatorApiConfig) -> dataflow_coordinator_grpc::CoordinatorApiClient {
    let env = sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect(format!("{}:{}", config.host, config.port).as_str());
    dataflow_coordinator_grpc::CoordinatorApiClient::new(channel)
}
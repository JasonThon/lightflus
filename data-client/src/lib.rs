use std::sync::Arc;

pub mod tableflow;
pub mod tableflow_grpc;

pub struct DataEngineConfig {
    pub host: String,
    pub port: usize,
}

pub fn new_data_engine_client(config: DataEngineConfig) -> tableflow_grpc::DataEngineClient {
    let env = Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect(format!("{}:{}", config.host, config.port).as_str());

    tableflow_grpc::DataEngineClient::new(channel)
}
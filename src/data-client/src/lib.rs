use std::sync::Arc;

pub mod tableflow;
pub mod tableflow_grpc;

pub struct DataEngineConfig {
    pub host: Option<String>,
    pub port: Option<usize>,
    pub uri: Option<String>,
}

impl DataEngineConfig {
    fn get_uri(&self) -> String {
        if self.uri.is_some() {
            self.uri.as_ref().unwrap().clone()
        } else if self.host.is_some() && self.port.is_some() {
            format!("{}:{}", self.host.as_ref().unwrap(), self.port.as_ref().unwrap())
        } else {
            "".to_string()
        }
    }
}

pub fn new_data_engine_client(config: DataEngineConfig) -> tableflow_grpc::DataEngineClient {
    let env = Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect(config.get_uri().as_str());

    tableflow_grpc::DataEngineClient::new(channel)
}
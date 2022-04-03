use std::sync;
use crate::dataflow_worker_grpc;

pub fn new_dataflow_worker_client(config: DataflowWorkerConfig) -> dataflow_worker_grpc::TaskWorkerApiClient {
    let env = sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect(config.to_uri().as_str());

    dataflow_worker_grpc::TaskWorkerApiClient::new(channel)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DataflowWorkerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub uri: Option<String>,
}

impl DataflowWorkerConfig {
    fn to_uri(&self) -> String {
        if self.uri.is_some() {
            self.uri.as_ref().unwrap().clone()
        } else {
            let host = self.host.as_ref().unwrap();
            let port = self.port.as_ref().unwrap();
            format!("{}:{}", host, port)
        }
    }
}
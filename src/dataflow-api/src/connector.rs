use std::sync;

use crate::dataflow_connector_grpc;

pub fn new_connector_client(conn_proxy: &String) -> dataflow_connector_grpc::ConnectorApiClient {
    let env = sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect(conn_proxy.as_str());

    dataflow_connector_grpc::ConnectorApiClient::new(channel)
}
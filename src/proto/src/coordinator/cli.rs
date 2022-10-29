use std::sync;

use super::coordinator_grpc::CoordinatorApiClient;

pub fn new_coordinator_client(uri: String) -> CoordinatorApiClient {
    let env = sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env).connect(uri.as_str());

    CoordinatorApiClient::new(channel)
}

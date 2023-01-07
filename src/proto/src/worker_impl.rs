use std::time::Duration;

use tonic::codegen::StdError;

use crate::worker::task_worker_api_client::TaskWorkerApiClient;

/// Extra implementation of [`TaskWorkerApiClient`]
impl TaskWorkerApiClient<tonic::transport::Channel> {
    /// Connect to remote task worker lazily with connection timeout
    pub fn with_connection_timeout<D>(dst: D, timeout: Duration) -> Self
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let conn = tonic::transport::Endpoint::new(dst)
            .expect("parse endpoint failed")
            .connect_timeout(timeout)
            .connect_lazy();
        Self::new(conn)
    }

    /// Try to connect to remote task worker with connection timeout
    pub async fn connect_with_timeout<D>(
        dst: D,
        timeout: Duration,
    ) -> Result<Self, tonic::transport::Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        match tonic::transport::Endpoint::new(dst) {
            Ok(endpoint) => endpoint
                .connect_timeout(timeout)
                .connect()
                .await
                .map(|channel| Self::new(channel)),
            Err(err) => Err(err),
        }
    }
}

pub enum ConnectionError {
    Timeout,
    Transport(tonic::transport::Error),
}

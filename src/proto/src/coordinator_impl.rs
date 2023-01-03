use std::time::Duration;

use tonic::codegen::StdError;

use crate::coordinator::coordinator_api_client::CoordinatorApiClient;

/// Extra implementation of [`CoordinatorApiClient`]
impl CoordinatorApiClient<tonic::transport::Channel> {
    /// Connect to remote coordinator lazily with connection timeout
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

    /// Try to connect to remote coordinator with connection timeout
    pub async fn connect_with_timeout<D>(
        dst: D,
        timeout: Duration,
    ) -> Result<Self, tonic::transport::Error>
    where
        D: TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let conn = tonic::transport::Endpoint::new(dst)?
            .connect_timeout(timeout)
            .connect()
            .await;
        conn.map(|channel| Self::new(channel))
    }
}

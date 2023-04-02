use crate::new_rpc_response;

use super::coord;
use proto::common::{Ack, Dataflow, DataflowStates, Heartbeat, ResourceId, Response};

use proto::coordinator::coordinator_api_server::CoordinatorApi;
use proto::coordinator::GetDataflowRequest;

use tonic::async_trait;

pub struct CoordinatorApiImpl {
    coordinator: coord::Coordinator,
}

impl CoordinatorApiImpl {
    pub fn new(coordinator: coord::Coordinator) -> CoordinatorApiImpl {
        CoordinatorApiImpl { coordinator }
    }
}

unsafe impl Send for CoordinatorApiImpl {}

unsafe impl Sync for CoordinatorApiImpl {}

#[async_trait]
impl CoordinatorApi for CoordinatorApiImpl {
    async fn receive_heartbeat(
        &self,
        request: tonic::Request<Heartbeat>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        self.coordinator.receive_heartbeart(request.get_ref()).await;
        Ok(tonic::Response::new(Response::ok()))
    }

    async fn receive_ack(
        &self,
        request: tonic::Request<Ack>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        self.coordinator.receive_ack(request.into_inner()).await;
        Ok(tonic::Response::new(Response::ok()))
    }

    async fn create_dataflow(
        &self,
        request: tonic::Request<Dataflow>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        self.coordinator
            .create_dataflow(request.into_inner())
            .await
            .map(|_| tonic::Response::new(Response::ok()))
    }
    async fn terminate_dataflow(
        &self,
        request: tonic::Request<ResourceId>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        self.coordinator
            .terminate_dataflow(request.get_ref())
            .await
            .map(|status| tonic::Response::new(Response::ok()))
    }
    async fn get_dataflow(
        &self,
        request: tonic::Request<GetDataflowRequest>,
    ) -> Result<tonic::Response<DataflowStates>, tonic::Status> {
        self.coordinator
            .get_dataflow(request.get_ref().job_id.as_ref().unwrap())
            .await
            .and_then(|dataflow| Ok(new_rpc_response(dataflow)))
    }
}

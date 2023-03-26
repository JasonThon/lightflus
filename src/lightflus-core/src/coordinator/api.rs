use super::coord;
use proto::common::{Ack, Dataflow, DataflowStatus, Heartbeat, ResourceId, Response, TaskInfo};

use proto::coordinator::coordinator_api_server::CoordinatorApi;
use proto::coordinator::{GetDataflowRequest, GetDataflowResponse};
use tokio::sync::RwLock;
use tonic::async_trait;

pub(crate) struct CoordinatorApiImpl {
    /// # TODO
    ///
    /// [`RwLock`] will block many `receive_heartbeat` and `receive_ack` requests if they are concurrently sent.
    /// To improve the performance, in next version, an implementation of concurrent [`std::collections::HashMap] will be added.
    /// Such map structure acquires only one lock on a single node which can minimize the scope of locking without locking the entire tree.
    coordinator: RwLock<coord::Coordinator>,
}

impl CoordinatorApiImpl {
    pub(crate) fn new(coordinator: coord::Coordinator) -> CoordinatorApiImpl {
        CoordinatorApiImpl {
            coordinator: RwLock::new(coordinator),
        }
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
        let mut write_lock = self.coordinator.write().await;
        write_lock.receive_heartbeart(request.get_ref()).await;
        Ok(tonic::Response::new(Response::ok()))
    }

    async fn report_task_info(
        &self,
        request: tonic::Request<TaskInfo>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        todo!()
    }

    async fn receive_ack(
        &self,
        request: tonic::Request<Ack>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock.receive_ack(request.into_inner()).await;
        Ok(tonic::Response::new(Response::ok()))
    }

    async fn create_dataflow(
        &self,
        request: tonic::Request<Dataflow>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock
            .create_dataflow(request.into_inner())
            .await
            .map(|_| tonic::Response::new(Response::ok()))
    }
    async fn terminate_dataflow(
        &self,
        request: tonic::Request<ResourceId>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock
            .terminate_dataflow(request.get_ref())
            .await
            .map(|status| tonic::Response::new(Response::ok()))
    }
    async fn get_dataflow(
        &self,
        request: tonic::Request<GetDataflowRequest>,
    ) -> Result<tonic::Response<GetDataflowResponse>, tonic::Status> {
        let read_lock = self.coordinator.read().await;
        match read_lock.get_dataflow(request.get_ref().job_id.as_ref().unwrap()) {
            Some(resp) => Ok(tonic::Response::new(GetDataflowResponse {
                status: DataflowStatus::Running as i32,
                graph: Some(resp),
            })),
            None => Err(tonic::Status::not_found(format!(
                "dataflow {:?} does not found",
                request.get_ref().job_id.as_ref().unwrap()
            ))),
        }
    }
}
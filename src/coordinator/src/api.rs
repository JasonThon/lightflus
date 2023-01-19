use crate::coord;
use proto::common::{Ack, Dataflow, DataflowStatus, Heartbeat, ResourceId, Response};

use proto::coordinator::coordinator_api_server::CoordinatorApi;
use proto::coordinator::{
    CreateDataflowResponse, GetDataflowRequest, GetDataflowResponse, TaskInfo,
    TerminateDataflowResponse,
};
use tokio::sync::RwLock;

pub(crate) struct CoordinatorApiImpl {
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

#[tonic::async_trait]
impl CoordinatorApi for CoordinatorApiImpl {
    async fn receive_heartbeat(
        &self,
        request: tonic::Request<Heartbeat>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock.receive_heartbeart(request.get_ref());
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
        todo!()
    }

    async fn create_dataflow(
        &self,
        request: tonic::Request<Dataflow>,
    ) -> Result<tonic::Response<CreateDataflowResponse>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock
            .create_dataflow(request.into_inner())
            .await
            .map(|_| {
                tonic::Response::new(CreateDataflowResponse {
                    status: DataflowStatus::Initialized as i32,
                })
            })
    }
    async fn terminate_dataflow(
        &self,
        request: tonic::Request<ResourceId>,
    ) -> Result<tonic::Response<TerminateDataflowResponse>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock
            .terminate_dataflow(request.get_ref())
            .await
            .map(|status| {
                tonic::Response::new(TerminateDataflowResponse {
                    status: status as i32,
                })
            })
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

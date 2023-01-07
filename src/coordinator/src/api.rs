use std::sync::Arc;

use crate::coord;
use proto::common::{Dataflow, DataflowStatus};
use proto::common::{ProbeRequest, ProbeResponse};
use proto::coordinator::coordinator_api_server::CoordinatorApi;
use proto::coordinator::{
    CreateDataflowResponse, GetDataflowRequest, GetDataflowResponse, TerminateDataflowRequest,
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
    async fn probe(
        &self,
        _request: tonic::Request<ProbeRequest>,
    ) -> Result<tonic::Response<ProbeResponse>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock.probe_state().await;

        Ok(tonic::Response::new(ProbeResponse {
            memory: 1.0,
            cpu: 1.0,
            available: true,
        }))
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
        request: tonic::Request<TerminateDataflowRequest>,
    ) -> Result<tonic::Response<TerminateDataflowResponse>, tonic::Status> {
        let mut write_lock = self.coordinator.write().await;
        write_lock
            .terminate_dataflow(request.get_ref().job_id.as_ref().unwrap())
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

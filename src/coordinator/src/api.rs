use std::cell::RefCell;

use crate::coord;
use proto::common::{Dataflow, DataflowStatus};
use proto::common::{ProbeRequest, ProbeResponse};
use proto::coordinator::coordinator_api_server::CoordinatorApi;
use proto::coordinator::{
    CreateDataflowResponse, GetDataflowRequest, GetDataflowResponse, TerminateDataflowRequest,
    TerminateDataflowResponse,
};

#[derive(Clone)]
pub(crate) struct CoordinatorApiImpl {
    coordinator: RefCell<coord::Coordinator>,
}

impl CoordinatorApiImpl {
    pub(crate) fn new(coordinator: coord::Coordinator) -> CoordinatorApiImpl {
        CoordinatorApiImpl {
            coordinator: RefCell::new(coordinator),
        }
    }
}

unsafe impl Send for CoordinatorApiImpl {}

unsafe impl Sync for CoordinatorApiImpl {}

#[tonic::async_trait]
impl CoordinatorApi for CoordinatorApiImpl {
    async fn probe(
        &self,
        request: tonic::Request<ProbeRequest>,
    ) -> Result<tonic::Response<ProbeResponse>, tonic::Status> {
        self.coordinator.borrow_mut().probe_state();

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
        self.coordinator
            .borrow_mut()
            .create_dataflow(request.into_inner())
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
        self.coordinator
            .borrow_mut()
            .terminate_dataflow(request.get_ref().job_id.as_ref().unwrap())
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
        match self
            .coordinator
            .borrow()
            .get_dataflow(request.get_ref().job_id.as_ref().unwrap())
        {
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

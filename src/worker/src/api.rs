use std::sync;

use crate::manager::ExecutorManager;
use crate::worker as w;
use proto::common::DataflowStatus;
use proto::common::KeyedDataEvent;
use proto::common::ProbeRequest;
use proto::common::ProbeResponse;
use proto::common::ResourceId;
use proto::worker::task_worker_api_server::TaskWorkerApi;
use proto::worker::CreateSubDataflowRequest;
use proto::worker::CreateSubDataflowResponse;
use proto::worker::SendEventToOperatorResponse;
use proto::worker::StopDataflowResponse;

#[derive(Clone)]
pub(crate) struct TaskWorkerApiImpl {
    worker: sync::Arc<w::TaskWorker>,
}

unsafe impl Send for TaskWorkerApiImpl {}

unsafe impl Sync for TaskWorkerApiImpl {}

impl TaskWorkerApiImpl {
    pub(crate) fn new(worker: w::TaskWorker) -> TaskWorkerApiImpl {
        TaskWorkerApiImpl {
            worker: sync::Arc::new(worker),
        }
    }
}

#[tonic::async_trait]
impl TaskWorkerApi for TaskWorkerApiImpl {
    async fn probe(
        &self,
        _request: tonic::Request<ProbeRequest>,
    ) -> Result<tonic::Response<ProbeResponse>, tonic::Status> {
        Ok(tonic::Response::new(ProbeResponse {
            memory: 1.0,
            cpu: 1.0,
            available: true,
        }))
    }

    async fn send_event_to_operator(
        &self,
        request: tonic::Request<KeyedDataEvent>,
    ) -> Result<tonic::Response<SendEventToOperatorResponse>, tonic::Status> {
        self.worker
            .send_event_to_operator(&request.get_ref())
            .await
            .map(|status| {
                tonic::Response::new(SendEventToOperatorResponse {
                    status: status as i32,
                })
            })
            .map_err(|err| err.into_grpc_status())
    }

    async fn stop_dataflow(
        &self,
        request: tonic::Request<ResourceId>,
    ) -> Result<tonic::Response<StopDataflowResponse>, tonic::Status> {
        self.worker
            .stop_dataflow(request.get_ref())
            .await
            .map(|_| tonic::Response::new(StopDataflowResponse { resp: None }))
            .map_err(|err| err.into_grpc_status())
    }
    async fn create_sub_dataflow(
        &self,
        request: tonic::Request<CreateSubDataflowRequest>,
    ) -> Result<tonic::Response<CreateSubDataflowResponse>, tonic::Status> {
        match &request.get_ref().dataflow {
            Some(dataflow) => match &request.get_ref().job_id {
                Some(job_id) => self
                    .worker
                    .create_dataflow(&job_id, dataflow)
                    .await
                    .map(|_| {
                        tonic::Response::new(CreateSubDataflowResponse {
                            status: DataflowStatus::Initialized as i32,
                        })
                    })
                    .map_err(|err| err.into_grpc_status()),
                None => Ok(tonic::Response::new(CreateSubDataflowResponse {
                    status: DataflowStatus::Closed as i32,
                })),
            },
            None => Ok(tonic::Response::new(CreateSubDataflowResponse {
                status: DataflowStatus::Closed as i32,
            })),
        }
    }
}

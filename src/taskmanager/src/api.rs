use std::sync;

use crate::manager::ExecutorManager;
use crate::worker as w;
use proto::common::Ack;
use proto::common::DataflowStatus;
use proto::common::Heartbeat;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::common::Response;
use proto::taskmanager::task_manager_api_server::TaskManagerApi;
use proto::taskmanager::CreateSubDataflowRequest;
use proto::taskmanager::CreateSubDataflowResponse;
use proto::taskmanager::SendEventToOperatorResponse;
use proto::taskmanager::StopDataflowResponse;

#[derive(Clone)]
pub(crate) struct TaskManagerApiImpl {
    worker: sync::Arc<w::TaskWorker>,
}

unsafe impl Send for TaskManagerApiImpl {}

unsafe impl Sync for TaskManagerApiImpl {}

impl TaskManagerApiImpl {
    pub(crate) fn new(worker: w::TaskWorker) -> TaskManagerApiImpl {
        TaskManagerApiImpl {
            worker: sync::Arc::new(worker),
        }
    }
}

#[tonic::async_trait]
impl TaskManagerApi for TaskManagerApiImpl {
    async fn send_event_to_operator(
        &self,
        request: tonic::Request<KeyedDataEvent>,
    ) -> Result<tonic::Response<SendEventToOperatorResponse>, tonic::Status> {
        self.worker
            .send_event_to_operator(request.get_ref())
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

    async fn receive_heartbeat(
        &self,
        request: tonic::Request<Heartbeat>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        Ok(tonic::Response::new(Response::ok()))
    }

    async fn receive_ack(
        &self,
        request: tonic::Request<Ack>,
    ) -> Result<tonic::Response<Response>, tonic::Status> {
        Ok(tonic::Response::new(Response::ok()))
    }
}

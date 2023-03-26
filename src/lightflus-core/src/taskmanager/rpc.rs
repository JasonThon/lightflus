use crossbeam_skiplist::SkipMap;
use proto::{
    common::{
        Ack, DataflowStatus, SubDataflowId, Heartbeat, KeyedDataEvent, KeyedEventSet, ResourceId,
        Response,
    },
    taskmanager::{
        task_manager_api_server::{TaskManagerApi, TaskManagerApiServer},
        BatchSendEventsToOperatorResponse, CreateSubDataflowRequest, CreateSubDataflowResponse,
        SendEventToOperatorResponse, StopDataflowResponse,
    },
};

use tonic::async_trait;

use crate::taskmanager::taskworker::{TaskWorker, TaskWorkerBuilder};

type RpcResponse<T> = Result<tonic::Response<T>, tonic::Status>;
type RpcRequest<T> = tonic::Request<T>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TaskManagerBuilder {
    // port of TaskManager
    pub port: usize,
    // max available number of jobs
    pub max_job_nums: usize,
}

impl TaskManagerBuilder {
    pub fn build(&self) -> TaskManagerApiServer<TaskManager> {
        let workers = SkipMap::new();
        TaskManagerApiServer::new(TaskManager {
            workers,
            job_id_map_execution_id: SkipMap::new(),
        })
    }
}

pub struct TaskManager {
    workers: SkipMap<SubDataflowId, TaskWorker>,
    job_id_map_execution_id: SkipMap<ResourceId, SubDataflowId>,
}

#[async_trait]
impl TaskManagerApi for TaskManager {
    async fn send_event_to_operator(
        &self,
        request: RpcRequest<KeyedDataEvent>,
    ) -> RpcResponse<SendEventToOperatorResponse> {
        let event = request.into_inner();
        match event.get_job_id_opt_ref() {
            Some(job_id) => match self.job_id_map_execution_id.get(job_id) {
                Some(execution_id) => match self.workers.get(execution_id.value()) {
                    Some(worker) => worker
                        .value()
                        .send_event_to_operator(event)
                        .await
                        .map(|status| {
                            tonic::Response::new(SendEventToOperatorResponse {
                                status: status as i32,
                            })
                        })
                        .map_err(|err| err.into_grpc_status()),
                    None => Err(tonic::Status::invalid_argument("no valid worker found")),
                },
                None => Err(tonic::Status::invalid_argument("no execution_id provided")),
            },
            None => Err(tonic::Status::invalid_argument("no execution_id provided")),
        }
    }

    async fn stop_dataflow(
        &self,
        request: RpcRequest<ResourceId>,
    ) -> RpcResponse<StopDataflowResponse> {
        match self.job_id_map_execution_id.get(request.get_ref()) {
            Some(entry) => {
                match self.workers.remove(entry.value()) {
                    Some(entry) => {
                        entry.remove();
                    }
                    None => {}
                }

                RpcResponse::Ok(tonic::Response::new(StopDataflowResponse::default()))
            }
            None => RpcResponse::Ok(tonic::Response::new(StopDataflowResponse::default())),
        }
    }

    async fn create_sub_dataflow(
        &self,
        request: RpcRequest<CreateSubDataflowRequest>,
    ) -> RpcResponse<CreateSubDataflowResponse> {
        match request.into_inner().dataflow.as_ref() {
            Some(dataflow) => {
                for execution_id in dataflow.get_execution_id_ref().iter() {
                    match self.workers.remove(*execution_id) {
                        Some(entry) => {
                            entry.remove();
                            let worker_builder = TaskWorkerBuilder::new(dataflow);
                            let worker = worker_builder.build();
                            let result = worker.await;
                            match result {
                                Ok(worker) => {
                                    self.workers.insert((*execution_id).clone(), worker);
                                    self.job_id_map_execution_id
                                        .insert(dataflow.get_job_id(), (*execution_id).clone());
                                }
                                Err(err) => return Err(err.into_grpc_status()),
                            };
                        }
                        None => {
                            return Err(tonic::Status::invalid_argument("no execution_id provided"))
                        }
                    }
                }
                Ok(tonic::Response::new(CreateSubDataflowResponse {
                    status: DataflowStatus::Initialized as i32,
                }))
            }
            None => Ok(tonic::Response::new(CreateSubDataflowResponse {
                status: DataflowStatus::Closed as i32,
            })),
        }
    }

    async fn receive_heartbeat(&self, request: RpcRequest<Heartbeat>) -> RpcResponse<Response> {
        let heartbeat = request.into_inner();
        match heartbeat.get_execution_id() {
            Some(execution_id) => {
                for entry in self.workers.get(execution_id).iter() {
                    let worker = entry.value();
                    worker.receive_heartbeat(&heartbeat)
                }

                Ok(tonic::Response::new(Response::ok()))
            }
            None => Err(tonic::Status::invalid_argument("no execution_id provided")),
        }
    }

    async fn receive_ack(&self, request: RpcRequest<Ack>) -> RpcResponse<Response> {
        let ack = request.into_inner();
        match ack.get_execution_id() {
            Some(execution_id) => {
                for entry in self.workers.get(execution_id).iter() {
                    let worker = entry.value();
                    worker.receive_ack(&ack)
                }

                Ok(tonic::Response::new(Response::ok()))
            }
            None => Err(tonic::Status::invalid_argument("no execution_id provided")),
        }
    }

    async fn batch_send_events_to_operator(
        &self,
        request: RpcRequest<KeyedEventSet>,
    ) -> RpcResponse<BatchSendEventsToOperatorResponse> {
        let event_set = request.into_inner();
        match event_set.job_id.as_ref() {
            Some(resource_id) => match self.job_id_map_execution_id.get(resource_id) {
                Some(execution_id) => {
                    match self.workers.get(execution_id.value()) {
                        Some(worker) => {
                            worker.value().batch_send_event_to_operator(event_set).await
                        },
                        None => {}
                    }
                }
                None => {}
            },
            None => Ok(tonic::Response::new(BatchSendEventsToOperatorResponse {})),
        }
        Ok(tonic::Response::new(BatchSendEventsToOperatorResponse {}))
    }
}

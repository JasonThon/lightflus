use std::collections::HashMap;

use crossbeam_skiplist::SkipMap;
use proto::{
    common::{Ack, DataflowStatus, ExecutionId, Heartbeat, KeyedDataEvent, ResourceId, Response},
    taskmanager::{
        task_manager_api_server::TaskManagerApi, CreateSubDataflowRequest,
        CreateSubDataflowResponse, SendEventToOperatorResponse, StopDataflowResponse,
    },
};
use tokio::sync::RwLock;
use tonic::async_trait;

use crate::taskworker::{TaskWorker, TaskWorkerBuilder};

type RpcResponse<T> = Result<tonic::Response<T>, tonic::Status>;
type RpcRequest<T> = tonic::Request<T>;

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TaskManagerBuilder {
    // port of TaskManager
    pub port: usize,
    // max available number of jobs
    pub max_job_nums: usize,
}

impl TaskManagerBuilder {
    pub fn build(&self) -> TaskManager {
        let workers = SkipMap::new();
        TaskManager {
            workers,
            job_id_map_execution_id: SkipMap::new(),
        }
    }
}

pub(crate) struct TaskManager {
    workers: SkipMap<ExecutionId, TaskWorker>,
    job_id_map_execution_id: SkipMap<ResourceId, ExecutionId>,
}

#[async_trait]
impl TaskManagerApi for TaskManager {
    async fn send_event_to_operator(
        &self,
        request: RpcRequest<KeyedDataEvent>,
    ) -> RpcResponse<SendEventToOperatorResponse> {
        let event = request.into_inner();
        match event.get_job_id_opt_ref() {
            Some(job_id) => match self.job_id_map_execution_id.read().await.get(job_id) {
                Some(execution_id) => match self.workers.get(execution_id) {
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
        match self.workers.get(request.get_ref()) {
            Some(entry) => {
                let worker = entry.value();
                worker.terminate_execution().await;
                entry.remove();
                RpcResponse::Ok(StopDataflowResponse::default())
            }
            None => RpcResponse::Ok(StopDataflowResponse::default()),
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
                            let worker = entry.value();
                            let closer = worker.terminate_execution();
                            let worker_builder = TaskWorkerBuilder::new(dataflow);
                            let worker = worker_builder.build();
                            let (_, worker) = tokio::join!(closer, worker);
                            match worker {
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
                    worker.write().await.receive_heartbeat(&heartbeat)
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
}

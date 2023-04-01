use std::fs;

use common::utils;
use crossbeam_skiplist::SkipMap;
use proto::{
    common::{
        Ack, DataflowStatus, Heartbeat, KeyedDataEvent, KeyedEventSet, ResourceId, Response,
        SubDataflowId, SubDataflowStates,
    },
    taskmanager::{
        task_manager_api_server::{TaskManagerApi, TaskManagerApiServer},
        BatchSendEventsToOperatorResponse, CreateSubDataflowRequest, CreateSubDataflowResponse,
        SendEventToOperatorResponse, StopDataflowResponse,
    },
};

use tonic::async_trait;

use crate::{
    errors::taskmanager::{execution_id_unprovided, no_found_worker, resource_id_unprovided},
    new_rpc_response,
    taskmanager::taskworker::{TaskWorker, TaskWorkerBuilder},
    RpcRequest, RpcResponse,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TaskManagerBuilder {
    // port of TaskManager
    pub port: usize,
    // max available number of jobs
    pub max_job_nums: usize,
}

pub fn load_builder() -> TaskManagerBuilder {
    serde_json::from_str::<TaskManagerBuilder>(
        common::utils::from_reader(
            fs::File::open(
                utils::Args::default()
                    .arg("c")
                    .map(|arg| arg.value.clone())
                    .unwrap_or("src/taskmanager/etc/taskmanager.json".to_string()),
            )
            .expect("config file open failed: "),
        )
        .expect("config file read failed: ")
        .as_str(),
    )
    .expect("config file parse failed: ")
}

impl TaskManagerBuilder {
    pub fn build(&self) -> TaskManagerApiServer<TaskManager> {
        let workers = SkipMap::new();
        TaskManagerApiServer::new(TaskManager { workers })
    }
}

pub struct TaskManager {
    workers: SkipMap<ResourceId, TaskWorker>,
}

#[async_trait]
impl TaskManagerApi for TaskManager {
    async fn send_event_to_operator(
        &self,
        request: RpcRequest<KeyedDataEvent>,
    ) -> RpcResponse<SendEventToOperatorResponse> {
        let event = request.into_inner();
        match event
            .get_job_id_opt_ref()
            .and_then(|job_id| self.workers.get(job_id))
        {
            Some(worker) => worker
                .value()
                .send_event_to_operator(event)
                .await
                .map(|status| {
                    new_rpc_response(SendEventToOperatorResponse {
                        status: status as i32,
                    })
                })
                .map_err(|err| err.into_grpc_status()),
            None => Err(no_found_worker().into_tonic_status()),
        }
    }

    async fn stop_dataflow(
        &self,
        request: RpcRequest<ResourceId>,
    ) -> RpcResponse<StopDataflowResponse> {
        match self.workers.remove(request.get_ref()) {
            Some(entry) => {
                entry.remove();
            }
            None => {}
        };
        Ok(new_rpc_response(StopDataflowResponse::default()))
    }

    async fn create_sub_dataflow(
        &self,
        request: RpcRequest<CreateSubDataflowRequest>,
    ) -> RpcResponse<CreateSubDataflowResponse> {
        let request = request.into_inner();
        let opt = request.dataflow.as_ref();
        opt.and_then(|dataflow| dataflow.job_id.as_ref())
            .and_then(|job_id| self.workers.remove(job_id))
            .iter()
            .for_each(|entry| {
                entry.remove();
            });
        match opt {
            Some(dataflow) => {
                let worker_builder = TaskWorkerBuilder::new(dataflow);
                match worker_builder.build().await {
                    Ok(worker) => {
                        match dataflow.job_id.as_ref() {
                            Some(job_id) => {
                                self.workers.insert(job_id.clone(), worker);
                            }
                            None => {}
                        };

                        Ok(new_rpc_response(CreateSubDataflowResponse {
                            status: DataflowStatus::Initialized as i32,
                        }))
                    }
                    Err(err) => Err(err.into_grpc_status()),
                }
            }
            None => Err(resource_id_unprovided().into_tonic_status()),
        }
    }

    async fn receive_heartbeat(&self, request: RpcRequest<Heartbeat>) -> RpcResponse<Response> {
        let heartbeat = request.into_inner();
        match heartbeat
            .get_subdataflow_id()
            .and_then(|subdataflow_id| subdataflow_id.job_id.as_ref())
        {
            Some(job_id) => {
                for entry in self.workers.get(job_id).iter() {
                    let worker = entry.value();
                    worker.receive_heartbeat(&heartbeat)
                }

                Ok(new_rpc_response(Response::ok()))
            }
            None => Err(execution_id_unprovided().into_tonic_status()),
        }
    }

    async fn receive_ack(&self, request: RpcRequest<Ack>) -> RpcResponse<Response> {
        let ack = request.into_inner();
        if let Some(job_id) = ack
            .get_execution_id()
            .and_then(|subdataflow_id| subdataflow_id.job_id.as_ref())
        {
            match self.workers.get(job_id) {
                Some(entry) => {
                    let worker = entry.value();
                    worker.receive_ack(&ack)
                }
                None => {}
            };
            Ok(new_rpc_response(Response::ok()))
        } else {
            Err(execution_id_unprovided().into_tonic_status())
        }
    }

    async fn batch_send_events_to_operator(
        &self,
        request: RpcRequest<KeyedEventSet>,
    ) -> RpcResponse<BatchSendEventsToOperatorResponse> {
        let event_set = request.into_inner();
        match event_set
            .job_id
            .as_ref()
            .and_then(|resource_id| self.workers.get(resource_id))
        {
            Some(worker) => worker
                .value()
                .batch_send_event_to_operator(event_set)
                .await
                .map(|_status| new_rpc_response(BatchSendEventsToOperatorResponse {}))
                .map_err(|err| err.into_grpc_status()),
            None => Ok(new_rpc_response(BatchSendEventsToOperatorResponse {})),
        }
    }
    async fn get_sub_dataflow(
        &self,
        request: RpcRequest<ResourceId>,
    ) -> RpcResponse<SubDataflowStates> {
        match self.workers.get(request.get_ref()) {
            Some(entry) => Ok(new_rpc_response(SubDataflowStates {
                subdataflow_infos: Some(entry.value().get_state().await),
            })),
            None => Err(no_found_worker().into_tonic_status()),
        }
    }
}

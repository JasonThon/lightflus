use std::sync;
use log::log;

use dataflow::{event, runtime, types, worker};
use dataflow::err::Error;
use dataflow_api::{dataflow_worker, probe};
use dataflow_api::dataflow_worker_grpc;

#[derive(Clone)]
pub(crate) struct TaskWorkerApiImpl {
    worker: sync::Arc<sync::Mutex<worker::TaskWorker>>,
}

unsafe impl Send for TaskWorkerApiImpl {}

unsafe impl Sync for TaskWorkerApiImpl {}

impl TaskWorkerApiImpl {
    pub(crate) fn new(worker: worker::TaskWorker) -> TaskWorkerApiImpl {
        TaskWorkerApiImpl {
            worker: sync::Arc::new(sync::Mutex::new(worker)),
        }
    }
}

impl dataflow_worker_grpc::TaskWorkerApi for TaskWorkerApiImpl {
    fn submit_action(&mut self,
                     ctx: ::grpcio::RpcContext,
                     _req: dataflow_worker::ActionSubmitRequest,
                     sink: ::grpcio::UnarySink<dataflow_worker::ActionSubmitResponse>) {
        let result = serde_json::from_slice::<event::GraphEvent>(_req.get_value());
        match result {
            Ok(event) => match event {
                event::GraphEvent::ExecutionGraphSubmit {
                    job_id, ops
                } => {
                    let mut w = self.worker.lock().unwrap();
                    (*w).build_new_graph(job_id, runtime::to_execution_graph(ops));
                    drop(w);

                    let mut response = dataflow_worker::ActionSubmitResponse::new();
                    response.set_code(200);
                    response.set_message("build graph success".to_string());
                    sink.success(response);
                }
                event::GraphEvent::NodeEventSubmit(ope) => {
                    let mut w = self.worker.lock().unwrap();
                    match (*w).submit_event(ope) {
                        Ok(_) => {
                            drop(w);

                            let mut response = dataflow_worker::ActionSubmitResponse::new();
                            response.set_code(200);
                            response.set_message("event submit successfully".to_string());
                            sink.success(response);
                        }
                        Err(err) => {
                            drop(w);
                            log::error!("submit event failed: {:?}", err);
                            sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::INTERNAL));
                        }
                    }
                }
                event::GraphEvent::StopGraph { job_id } => {
                    log::debug!("start stopping job {:?}", &job_id);
                    let mut w = self.worker.lock().unwrap();
                    match (*w).stop_job(job_id) {
                        Ok(_) => {
                            drop(w);

                            let mut response = dataflow_worker::ActionSubmitResponse::new();
                            response.set_code(200);
                            response.set_message("stop job successful".to_string());
                            sink.success(response);
                        }
                        Err(err) => {
                            drop(w);
                            log::error!("stop job failed: {:?}", err);
                            sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::INTERNAL));
                        }
                    }
                }
            },
            Err(err) => {
                log::error!("submit event failed: {:?}", err);
                sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::INTERNAL));
            }
        }
    }

    fn probe(&mut self,
             ctx: grpcio::RpcContext,
             _req: probe::ProbeRequest,
             sink: grpcio::UnarySink<probe::ProbeResponse>) {
        let mut response = probe::ProbeResponse::new();
        response.set_available(true);
        match _req.probeType {
            probe::ProbeRequest_ProbeType::Liveness => {
                sink.success(response);
            }
            probe::ProbeRequest_ProbeType::Readiness => {
                sink.success(response);
            }
        }
    }
}
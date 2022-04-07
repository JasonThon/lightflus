use std::sync;

use dataflow::{event, worker};
use dataflow::err::Error;
use dataflow_api::{dataflow_worker, probe};
use dataflow_api::dataflow_worker_grpc;

#[derive(Clone)]
pub(crate) struct TaskWorkerApiImpl {
    worker: sync::Arc<actix::Addr<worker::TaskWorker>>,
}

unsafe impl Send for TaskWorkerApiImpl {}

unsafe impl Sync for TaskWorkerApiImpl {}

impl TaskWorkerApiImpl {
    pub(crate) fn new(worker: actix::Addr<worker::TaskWorker>) -> TaskWorkerApiImpl {
        TaskWorkerApiImpl {
            worker: sync::Arc::new(worker),
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
            Ok(event) => {
                self.worker.do_send(event);
                let mut response = dataflow_worker::ActionSubmitResponse::new();
                response.set_code(200);
                response.set_message("success".to_string());
                sink.success(response);
            }
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
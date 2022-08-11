use std::{collections, sync};

use common::event;
use common::err::{Error, TaskWorkerError};
use proto::common::probe;
use proto::worker::worker;
use proto::worker::worker_grpc;
use crate::worker as w;

#[derive(Clone)]
pub(crate) struct TaskWorkerApiImpl {
    worker: sync::Arc<w::TaskWorker>,
}

unsafe impl Send for TaskWorkerApiImpl {}

unsafe impl Sync for TaskWorkerApiImpl {}

impl TaskWorkerApiImpl {
    pub(crate) fn new(worker: w::TaskWorker) -> TaskWorkerApiImpl {
        TaskWorkerApiImpl {
            worker: sync::Arc::new(worker)
        }
    }
}

impl worker_grpc::TaskWorkerApi for TaskWorkerApiImpl {
    fn probe(&mut self,
             _ctx: grpcio::RpcContext,
             req: probe::ProbeRequest,
             sink: grpcio::UnarySink<probe::ProbeResponse>) {
        let mut response = probe::ProbeResponse::new();
        response.available = true;
        match req.probeType {
            probe::ProbeRequest_ProbeType::Liveness => {
                sink.success(response);
            }
            probe::ProbeRequest_ProbeType::Readiness => {
                sink.success(response);
            }
        }
    }

    fn dispatch_data_events(&mut self, _ctx: grpcio::RpcContext, req: worker::DispatchDataEventsRequest, sink: grpcio::UnarySink<worker::DispatchDataEventsResponse>) {
        match self.worker.dispatch_events(req.events.to_vec()) {
            Ok(status_set) => {
                let mut response = worker::DispatchDataEventsResponse::new();
                response.set_statusSet(
                    status_set.iter()
                        .map(|(key, status)| (key.clone(), status.clone()))
                        .collect()
                );

                sink.success(response);
            }

            Err(err) => {
                sink.fail(grpcio::RpcStatus::with_message(
                    grpcio::RpcStatusCode::INTERNAL,
                    format!("{:?}", err),
                ));
            }
        }
    }

    fn stop_dataflow(&mut self, ctx: grpcio::RpcContext, req: worker::StopDataflowRequest, sink: grpcio::UnarySink<worker::StopDataflowResponse>) {
        if ctx.deadline().exceeded() {
            sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::DEADLINE_EXCEEDED));
        } else {
            match self.worker.stop_dataflow(req.job_id.unwrap()) {
                Ok(_) => sink.success(worker::StopDataflowResponse::default()),
                Err(err) => sink.fail(grpcio::RpcStatus::with_message(
                    grpcio::RpcStatusCode::INTERNAL,
                    format!("{:?}", err),
                ))
            };
        }
    }

    fn create_dataflow(&mut self, ctx: grpcio::RpcContext, req: worker::CreateDataflowRequest, sink: grpcio::UnarySink<worker::CreateDataflowResponse>) {
        if ctx.deadline().exceeded() {
            sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::DEADLINE_EXCEEDED));
        } else {
            match self.worker.create_dataflow(req.job_id.unwrap(), req.dataflow.unwrap()) {
                Ok(_) => {
                    sink.success(worker::CreateDataflowResponse::default());
                }
                Err(err) => {
                    sink.fail(grpcio::RpcStatus::with_message(
                        grpcio::RpcStatusCode::INTERNAL,
                        format!("{:?}", err),
                    ));
                }
            }
        }
    }
}
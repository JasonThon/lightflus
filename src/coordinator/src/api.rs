use crate::coord;
use common::{err::Error, net::cluster};
use grpcio::{RpcContext, RpcStatus, RpcStatusCode, UnarySink};
use proto::common::probe;
use proto::common::probe::{ProbeRequest, ProbeResponse};
use proto::common::stream::{Dataflow, DataflowStatus};
use proto::coordinator::coordinator::{
    CreateStreamGraphResponse, GetDataflowRequest, GetDataflowResponse, TerminateDataflowRequest,
    TerminateDataflowResponse,
};
use proto::coordinator::coordinator_grpc::CoordinatorApi;
use std::sync;

#[derive(Clone)]
pub(crate) struct CoordinatorApiImpl {
    coordinator: coord::Coordinator,
    cluster: sync::Arc<sync::RwLock<cluster::Cluster>>,
}

impl CoordinatorApiImpl {
    pub(crate) fn new(
        coordinator: coord::Coordinator,
        cluster: cluster::Cluster,
    ) -> CoordinatorApiImpl {
        CoordinatorApiImpl {
            coordinator: coordinator,
            cluster: sync::Arc::new(sync::RwLock::new(cluster)),
        }
    }
}

unsafe impl Send for CoordinatorApiImpl {}

unsafe impl Sync for CoordinatorApiImpl {}

impl CoordinatorApi for CoordinatorApiImpl {
    fn probe(&mut self, _ctx: RpcContext, req: ProbeRequest, sink: UnarySink<ProbeResponse>) {
        match req.probeType {
            probe::ProbeRequest_ProbeType::Readiness => match self.cluster.try_write() {
                Ok(mut cluster) => {
                    sink.success(ProbeResponse::default());
                    cluster.probe_state();
                }
                Err(_) => {
                    sink.success(ProbeResponse::default());
                }
            },
            probe::ProbeRequest_ProbeType::Liveness => {
                sink.success(ProbeResponse::default());
            }
        }
    }

    fn create_dataflow(
        &mut self,
        _ctx: RpcContext,
        req: Dataflow,
        sink: UnarySink<CreateStreamGraphResponse>,
    ) {
        match self.coordinator.create_dataflow(req) {
            Ok(_) => {
                let ref mut resp = CreateStreamGraphResponse::default();
                resp.set_status(DataflowStatus::RUNNING);
                sink.success(resp.clone());
            }
            Err(err) => {
                let status = grpcio::RpcStatus::with_message(
                    grpcio::RpcStatusCode::from(err.code),
                    err.msg.clone(),
                );
                sink.fail(status);
            }
        }
    }

    fn terminate_dataflow(
        &mut self,
        _ctx: RpcContext,
        _req: TerminateDataflowRequest,
        sink: UnarySink<TerminateDataflowResponse>,
    ) {
        match self.coordinator.terminate_dataflow(_req.get_job_id()) {
            Ok(status) => {
                let mut resp = TerminateDataflowResponse::default();
                resp.set_status(status);
                sink.success(resp);
            }
            Err(err) => {
                sink.fail(RpcStatus::with_message(
                    RpcStatusCode::from(err.code),
                    err.msg,
                ));
            }
        }
    }

    fn get_dataflow(
        &mut self,
        _ctx: RpcContext,
        _req: GetDataflowRequest,
        sink: UnarySink<GetDataflowResponse>,
    ) {
        match self
            .coordinator
            .get_dataflow(_req.get_job_id())
            .map(|dataflow| {
                let mut resp = GetDataflowResponse::default();
                resp.set_graph(dataflow.clone());
                resp
            }) {
            Some(resp) => {
                sink.success(resp);
            }
            None => {
                sink.fail(RpcStatus::new(RpcStatusCode::NOT_FOUND));
            }
        }
    }
}

use common::event;
use crate::{cluster, coord};
use common::err::Error;
use std::sync;
use grpcio::{RpcContext, UnarySink};
use proto::common::probe;
use proto::common::probe::{ProbeRequest, ProbeResponse};
use proto::common::stream::Dataflow;
use proto::coordinator::coordinator::{CreateStreamGraphResponse, GetDataflowRequest, GetDataflowResponse, TerminateDataflowRequest, TerminateDataflowResponse};
use proto::coordinator::coordinator_grpc::CoordinatorApi;

const SUCCESS_MSG: &str = "success";

#[derive(Clone)]
pub(crate) struct CoordinatorApiImpl {
    coordinator: sync::Arc<coord::Coordinator>,
    cluster: sync::Arc<sync::RwLock<cluster::Cluster>>,
}

impl CoordinatorApiImpl {
    pub(crate) fn new(coordinator: coord::Coordinator, cluster: cluster::Cluster) -> CoordinatorApiImpl {
        CoordinatorApiImpl {
            coordinator: sync::Arc::new(coordinator),
            cluster: sync::Arc::new(sync::RwLock::new(cluster)),
        }
    }
}

unsafe impl Send for CoordinatorApiImpl {}

unsafe impl Sync for CoordinatorApiImpl {}

impl CoordinatorApi for CoordinatorApiImpl {
    fn probe(&mut self,
             _ctx: RpcContext,
             req: ProbeRequest,
             sink: UnarySink<ProbeResponse>) {
        match req.probeType.unwrap() {
            probe::ProbeRequest_ProbeType::Readiness => {
                match self.cluster.try_write() {
                    Ok(mut cluster) => {
                        sink.success(ProbeResponse::default());
                        cluster.probe_state();
                    }
                    Err(_) => {
                        sink.success(ProbeResponse::default());
                    }
                }
            }
            probe::ProbeRequest_ProbeType::Liveness => {
                sink.success(ProbeResponse::default());
            }
        }
    }

    fn create_dataflow(&mut self, ctx: RpcContext, _req: Dataflow, sink: UnarySink<CreateStreamGraphResponse>) {
        todo!()
    }

    fn terminate_dataflow(&mut self, ctx: RpcContext, _req: TerminateDataflowRequest, sink: UnarySink<TerminateDataflowResponse>) {
        todo!()
    }

    fn get_dataflow(&mut self, ctx: RpcContext, _req: GetDataflowRequest, sink: UnarySink<GetDataflowResponse>) {
        todo!()
    }
}
use common::proto::probe;
use dataflow_api::coordinator::coordinator_grpc;
use common::event;
use crate::{cluster, coord};
use common::err::Error;
use std::sync;
use grpcio::{RpcContext, UnarySink};
use dataflow_api::coordinator::coordinator;

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

impl coordinator_grpc::CoordinatorApi for CoordinatorApiImpl {
    fn probe(&mut self,
             _ctx: RpcContext,
             req: probe::ProbeRequest,
             sink: UnarySink<probe::ProbeResponse>) {
        match req.probeType.unwrap() {
            probe::probe_request::ProbeType::Readiness => {
                match self.cluster.try_write() {
                    Ok(mut cluster) => {
                        sink.success(probe::ProbeResponse::default());
                        cluster.probe_state();
                    }
                    Err(_) => {
                        sink.success(probe::ProbeResponse::default());
                    }
                }
            }
            probe::probe_request::ProbeType::Liveness => {
                sink.success(probe::ProbeResponse::default());
            }
        }
    }

    fn create_stream_graph(&mut self,
                           _ctx: RpcContext,
                           _req: common::proto::stream::StreamGraph,
                           sink: UnarySink<coordinator::CreateStreamGraphResponse>) {

    }

    fn terminate_stream_graph(&mut self, ctx: RpcContext,
                              _req: coordinator::TerminateStreamGraphRequest,
                              sink: UnarySink<coordinator::TerminateStreamGraphResponse>) {
        todo!()
    }

    fn get_stream_graph(&mut self, ctx: RpcContext,
                        _req: coordinator::GetStreamGraphRequest,
                        sink: UnarySink<coordinator::GetStreamGraphResponse>) {
        todo!()
    }
}
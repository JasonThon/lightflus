use dataflow_api::dataflow_coordinator;
use dataflow_api::dataflow_coordinator_grpc;
use dataflow_api::probe;

pub(crate) struct ConnectorApiImpl {
}

impl dataflow_coordinator_grpc::CoordinatorApi for ConnectorApiImpl {
    fn handle_event(&mut self,
                    ctx: ::grpcio::RpcContext,
                    _req: dataflow_coordinator::EventRequest,
                    sink: ::grpcio::UnarySink<dataflow_coordinator::EventResponse>) {
        todo!()
    }

    fn probe(&mut self,
             ctx: grpcio::RpcContext,
             _req: probe::ProbeRequest,
             sink: grpcio::UnarySink<probe::ProbeResponse>) {
        todo!()
    }
}
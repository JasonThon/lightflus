use proto::storage::storage_grpc;

#[derive(Clone)]
pub struct QueryEngineApiImpl {}

impl QueryEngineApiImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl storage_grpc::QueryEngine for QueryEngineApiImpl {
    fn probe(
        &mut self,
        ctx: grpcio::RpcContext,
        _req: proto::common::probe::ProbeRequest,
        sink: grpcio::UnarySink<proto::common::probe::ProbeResponse>,
    ) {
        grpcio::unimplemented_call!(ctx, sink)
    }

    fn query(
        &mut self,
        ctx: grpcio::RpcContext,
        _req: proto::storage::storage::QueryRequest,
        sink: grpcio::UnarySink<proto::storage::storage::QueryResponse>,
    ) {
        grpcio::unimplemented_call!(ctx, sink)
    }

    fn subscribe_change_stream(
        &mut self,
        ctx: grpcio::RpcContext,
        _req: proto::storage::storage::SubscribeChangeStreamRequest,
        sink: grpcio::UnarySink<proto::storage::storage::SubscribeChangeStreamResponse>,
    ) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

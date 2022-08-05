use grpcio::{RpcContext, UnarySink};
use proto::common::common::Response;
use proto::qe::qe::{CreateSchemaRequest, DeleteRequest, DeleteSchemaRequest, GetSchemaRequest, GetSchemaResponse, InsertRequest, QueryRequest, QueryResponse, UpdateRequest, UpdateResponse, UpdateSchemaRequest};

pub struct QueryEngine {

}

impl proto::qe::qe_grpc::QueryEngine for QueryEngine {
    fn query(&mut self, ctx: RpcContext, _req: QueryRequest, sink: UnarySink<QueryResponse>) {
        todo!()
    }

    fn delete(&mut self, ctx: RpcContext, _req: DeleteRequest, sink: UnarySink<Response>) {
        todo!()
    }

    fn update(&mut self, ctx: RpcContext, _req: UpdateRequest, sink: UnarySink<UpdateResponse>) {
        todo!()
    }

    fn insert(&mut self, ctx: RpcContext, _req: InsertRequest, sink: UnarySink<Response>) {
        todo!()
    }

    fn get_schema(&mut self, ctx: RpcContext, _req: GetSchemaRequest, sink: UnarySink<GetSchemaResponse>) {
        todo!()
    }

    fn delete_schema(&mut self, ctx: RpcContext, _req: DeleteSchemaRequest, sink: UnarySink<Response>) {
        todo!()
    }

    fn update_schema(&mut self, ctx: RpcContext, _req: UpdateSchemaRequest, sink: UnarySink<Response>) {
        todo!()
    }

    fn create_schema(&mut self, ctx: RpcContext, _req: CreateSchemaRequest, sink: UnarySink<Response>) {
        todo!()
    }
}
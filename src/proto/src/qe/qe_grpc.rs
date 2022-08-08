// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use crate::common::common;

const METHOD_QUERY_ENGINE_QUERY: ::grpcio::Method<super::qe::QueryRequest, super::qe::QueryResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/Query",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_DELETE: ::grpcio::Method<super::qe::DeleteRequest, common::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_UPDATE: ::grpcio::Method<super::qe::UpdateRequest, super::qe::UpdateResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/Update",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_INSERT: ::grpcio::Method<super::qe::InsertRequest, common::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/Insert",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_GET_SCHEMA: ::grpcio::Method<super::qe::GetSchemaRequest, super::qe::GetSchemaResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/GetSchema",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_DELETE_SCHEMA: ::grpcio::Method<super::qe::DeleteSchemaRequest, common::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/DeleteSchema",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_UPDATE_SCHEMA: ::grpcio::Method<super::qe::UpdateSchemaRequest, common::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/UpdateSchema",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_CREATE_SCHEMA: ::grpcio::Method<super::qe::CreateSchemaRequest, common::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.QueryEngine/CreateSchema",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct QueryEngineClient {
    client: ::grpcio::Client,
}

impl QueryEngineClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        QueryEngineClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn query_opt(&self, req: &super::qe::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::qe::QueryResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_QUERY, req, opt)
    }

    pub fn query(&self, req: &super::qe::QueryRequest) -> ::grpcio::Result<super::qe::QueryResponse> {
        self.query_opt(req, ::grpcio::CallOption::default())
    }

    pub fn query_async_opt(&self, req: &super::qe::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::QueryResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_QUERY, req, opt)
    }

    pub fn query_async(&self, req: &super::qe::QueryRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::QueryResponse>> {
        self.query_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::qe::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::Response> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::qe::DeleteRequest) -> ::grpcio::Result<common::Response> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::qe::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::qe::DeleteRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_opt(&self, req: &super::qe::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::qe::UpdateResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_UPDATE, req, opt)
    }

    pub fn update(&self, req: &super::qe::UpdateRequest) -> ::grpcio::Result<super::qe::UpdateResponse> {
        self.update_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_async_opt(&self, req: &super::qe::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::UpdateResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_UPDATE, req, opt)
    }

    pub fn update_async(&self, req: &super::qe::UpdateRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::UpdateResponse>> {
        self.update_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_opt(&self, req: &super::qe::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::Response> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_INSERT, req, opt)
    }

    pub fn insert(&self, req: &super::qe::InsertRequest) -> ::grpcio::Result<common::Response> {
        self.insert_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_async_opt(&self, req: &super::qe::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_INSERT, req, opt)
    }

    pub fn insert_async(&self, req: &super::qe::InsertRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.insert_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_schema_opt(&self, req: &super::qe::GetSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::qe::GetSchemaResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_GET_SCHEMA, req, opt)
    }

    pub fn get_schema(&self, req: &super::qe::GetSchemaRequest) -> ::grpcio::Result<super::qe::GetSchemaResponse> {
        self.get_schema_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_schema_async_opt(&self, req: &super::qe::GetSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::GetSchemaResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_GET_SCHEMA, req, opt)
    }

    pub fn get_schema_async(&self, req: &super::qe::GetSchemaRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::qe::GetSchemaResponse>> {
        self.get_schema_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_schema_opt(&self, req: &super::qe::DeleteSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::Response> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_DELETE_SCHEMA, req, opt)
    }

    pub fn delete_schema(&self, req: &super::qe::DeleteSchemaRequest) -> ::grpcio::Result<common::Response> {
        self.delete_schema_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_schema_async_opt(&self, req: &super::qe::DeleteSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_DELETE_SCHEMA, req, opt)
    }

    pub fn delete_schema_async(&self, req: &super::qe::DeleteSchemaRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.delete_schema_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_schema_opt(&self, req: &super::qe::UpdateSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::Response> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_UPDATE_SCHEMA, req, opt)
    }

    pub fn update_schema(&self, req: &super::qe::UpdateSchemaRequest) -> ::grpcio::Result<common::Response> {
        self.update_schema_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_schema_async_opt(&self, req: &super::qe::UpdateSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_UPDATE_SCHEMA, req, opt)
    }

    pub fn update_schema_async(&self, req: &super::qe::UpdateSchemaRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.update_schema_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_schema_opt(&self, req: &super::qe::CreateSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::Response> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_CREATE_SCHEMA, req, opt)
    }

    pub fn create_schema(&self, req: &super::qe::CreateSchemaRequest) -> ::grpcio::Result<common::Response> {
        self.create_schema_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_schema_async_opt(&self, req: &super::qe::CreateSchemaRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_CREATE_SCHEMA, req, opt)
    }

    pub fn create_schema_async(&self, req: &super::qe::CreateSchemaRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::Response>> {
        self.create_schema_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait QueryEngine {
    fn query(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::QueryRequest, sink: ::grpcio::UnarySink<super::qe::QueryResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn delete(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::DeleteRequest, sink: ::grpcio::UnarySink<common::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn update(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::UpdateRequest, sink: ::grpcio::UnarySink<super::qe::UpdateResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn insert(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::InsertRequest, sink: ::grpcio::UnarySink<common::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn get_schema(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::GetSchemaRequest, sink: ::grpcio::UnarySink<super::qe::GetSchemaResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn delete_schema(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::DeleteSchemaRequest, sink: ::grpcio::UnarySink<common::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn update_schema(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::UpdateSchemaRequest, sink: ::grpcio::UnarySink<common::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn create_schema(&mut self, ctx: ::grpcio::RpcContext, _req: super::qe::CreateSchemaRequest, sink: ::grpcio::UnarySink<common::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_query_engine<S: QueryEngine + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_QUERY, move |ctx, req, resp| {
        instance.query(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_UPDATE, move |ctx, req, resp| {
        instance.update(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_INSERT, move |ctx, req, resp| {
        instance.insert(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_GET_SCHEMA, move |ctx, req, resp| {
        instance.get_schema(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_DELETE_SCHEMA, move |ctx, req, resp| {
        instance.delete_schema(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_UPDATE_SCHEMA, move |ctx, req, resp| {
        instance.update_schema(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_CREATE_SCHEMA, move |ctx, req, resp| {
        instance.create_schema(ctx, req, resp)
    });
    builder.build()
}

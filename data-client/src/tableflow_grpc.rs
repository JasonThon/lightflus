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

const METHOD_DATA_ENGINE_QUERY: ::grpcio::Method<super::tableflow::QueryRequest, super::tableflow::QueryResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.DataEngine/Query",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_DATA_ENGINE_DELETE: ::grpcio::Method<super::tableflow::DeleteRequest, super::tableflow::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.DataEngine/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_DATA_ENGINE_UPDATE: ::grpcio::Method<super::tableflow::UpdateRequest, super::tableflow::UpdateResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.DataEngine/Update",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_DATA_ENGINE_INSERT: ::grpcio::Method<super::tableflow::InsertRequest, super::tableflow::InsertResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.DataEngine/Insert",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct DataEngineClient {
    client: ::grpcio::Client,
}

impl DataEngineClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        DataEngineClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn query_opt(&self, req: &super::tableflow::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::QueryResponse> {
        self.client.unary_call(&METHOD_DATA_ENGINE_QUERY, req, opt)
    }

    pub fn query(&self, req: &super::tableflow::QueryRequest) -> ::grpcio::Result<super::tableflow::QueryResponse> {
        self.query_opt(req, ::grpcio::CallOption::default())
    }

    pub fn query_async_opt(&self, req: &super::tableflow::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::QueryResponse>> {
        self.client.unary_call_async(&METHOD_DATA_ENGINE_QUERY, req, opt)
    }

    pub fn query_async(&self, req: &super::tableflow::QueryRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::QueryResponse>> {
        self.query_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::tableflow::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::Response> {
        self.client.unary_call(&METHOD_DATA_ENGINE_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::tableflow::DeleteRequest) -> ::grpcio::Result<super::tableflow::Response> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::tableflow::DeleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.client.unary_call_async(&METHOD_DATA_ENGINE_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::tableflow::DeleteRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_opt(&self, req: &super::tableflow::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::UpdateResponse> {
        self.client.unary_call(&METHOD_DATA_ENGINE_UPDATE, req, opt)
    }

    pub fn update(&self, req: &super::tableflow::UpdateRequest) -> ::grpcio::Result<super::tableflow::UpdateResponse> {
        self.update_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_async_opt(&self, req: &super::tableflow::UpdateRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::UpdateResponse>> {
        self.client.unary_call_async(&METHOD_DATA_ENGINE_UPDATE, req, opt)
    }

    pub fn update_async(&self, req: &super::tableflow::UpdateRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::UpdateResponse>> {
        self.update_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_opt(&self, req: &super::tableflow::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::InsertResponse> {
        self.client.unary_call(&METHOD_DATA_ENGINE_INSERT, req, opt)
    }

    pub fn insert(&self, req: &super::tableflow::InsertRequest) -> ::grpcio::Result<super::tableflow::InsertResponse> {
        self.insert_opt(req, ::grpcio::CallOption::default())
    }

    pub fn insert_async_opt(&self, req: &super::tableflow::InsertRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::InsertResponse>> {
        self.client.unary_call_async(&METHOD_DATA_ENGINE_INSERT, req, opt)
    }

    pub fn insert_async(&self, req: &super::tableflow::InsertRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::InsertResponse>> {
        self.insert_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait DataEngine {
    fn query(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::QueryRequest, sink: ::grpcio::UnarySink<super::tableflow::QueryResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn delete(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::DeleteRequest, sink: ::grpcio::UnarySink<super::tableflow::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn update(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::UpdateRequest, sink: ::grpcio::UnarySink<super::tableflow::UpdateResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn insert(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::InsertRequest, sink: ::grpcio::UnarySink<super::tableflow::InsertResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_data_engine<S: DataEngine + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_DATA_ENGINE_QUERY, move |ctx, req, resp| {
        instance.query(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_DATA_ENGINE_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_DATA_ENGINE_UPDATE, move |ctx, req, resp| {
        instance.update(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_DATA_ENGINE_INSERT, move |ctx, req, resp| {
        instance.insert(ctx, req, resp)
    });
    builder.build()
}

const METHOD_METADATA_ENGINE_GET: ::grpcio::Method<super::tableflow::TableQuery, super::tableflow::Table> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.MetadataEngine/Get",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_METADATA_ENGINE_DELETE: ::grpcio::Method<super::tableflow::TableQuery, super::tableflow::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.MetadataEngine/Delete",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_METADATA_ENGINE_UPDATE: ::grpcio::Method<super::tableflow::UpdateMetadataRequest, super::tableflow::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.MetadataEngine/Update",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_METADATA_ENGINE_CREATE: ::grpcio::Method<super::tableflow::Table, super::tableflow::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/tableflow.MetadataEngine/Create",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct MetadataEngineClient {
    client: ::grpcio::Client,
}

impl MetadataEngineClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        MetadataEngineClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn get_opt(&self, req: &super::tableflow::TableQuery, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::Table> {
        self.client.unary_call(&METHOD_METADATA_ENGINE_GET, req, opt)
    }

    pub fn get(&self, req: &super::tableflow::TableQuery) -> ::grpcio::Result<super::tableflow::Table> {
        self.get_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_async_opt(&self, req: &super::tableflow::TableQuery, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Table>> {
        self.client.unary_call_async(&METHOD_METADATA_ENGINE_GET, req, opt)
    }

    pub fn get_async(&self, req: &super::tableflow::TableQuery) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Table>> {
        self.get_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_opt(&self, req: &super::tableflow::TableQuery, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::Response> {
        self.client.unary_call(&METHOD_METADATA_ENGINE_DELETE, req, opt)
    }

    pub fn delete(&self, req: &super::tableflow::TableQuery) -> ::grpcio::Result<super::tableflow::Response> {
        self.delete_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_async_opt(&self, req: &super::tableflow::TableQuery, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.client.unary_call_async(&METHOD_METADATA_ENGINE_DELETE, req, opt)
    }

    pub fn delete_async(&self, req: &super::tableflow::TableQuery) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.delete_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_opt(&self, req: &super::tableflow::UpdateMetadataRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::Response> {
        self.client.unary_call(&METHOD_METADATA_ENGINE_UPDATE, req, opt)
    }

    pub fn update(&self, req: &super::tableflow::UpdateMetadataRequest) -> ::grpcio::Result<super::tableflow::Response> {
        self.update_opt(req, ::grpcio::CallOption::default())
    }

    pub fn update_async_opt(&self, req: &super::tableflow::UpdateMetadataRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.client.unary_call_async(&METHOD_METADATA_ENGINE_UPDATE, req, opt)
    }

    pub fn update_async(&self, req: &super::tableflow::UpdateMetadataRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.update_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_opt(&self, req: &super::tableflow::Table, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::tableflow::Response> {
        self.client.unary_call(&METHOD_METADATA_ENGINE_CREATE, req, opt)
    }

    pub fn create(&self, req: &super::tableflow::Table) -> ::grpcio::Result<super::tableflow::Response> {
        self.create_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_async_opt(&self, req: &super::tableflow::Table, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.client.unary_call_async(&METHOD_METADATA_ENGINE_CREATE, req, opt)
    }

    pub fn create_async(&self, req: &super::tableflow::Table) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::tableflow::Response>> {
        self.create_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait MetadataEngine {
    fn get(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::TableQuery, sink: ::grpcio::UnarySink<super::tableflow::Table>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn delete(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::TableQuery, sink: ::grpcio::UnarySink<super::tableflow::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn update(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::UpdateMetadataRequest, sink: ::grpcio::UnarySink<super::tableflow::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn create(&mut self, ctx: ::grpcio::RpcContext, _req: super::tableflow::Table, sink: ::grpcio::UnarySink<super::tableflow::Response>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_metadata_engine<S: MetadataEngine + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_METADATA_ENGINE_GET, move |ctx, req, resp| {
        instance.get(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_METADATA_ENGINE_DELETE, move |ctx, req, resp| {
        instance.delete(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_METADATA_ENGINE_UPDATE, move |ctx, req, resp| {
        instance.update(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_METADATA_ENGINE_CREATE, move |ctx, req, resp| {
        instance.create(ctx, req, resp)
    });
    builder.build()
}

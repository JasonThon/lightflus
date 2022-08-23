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
use common::probe;

use crate::common;

const METHOD_QUERY_ENGINE_PROBE: ::grpcio::Method<probe::ProbeRequest, probe::ProbeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.QueryEngine/Probe",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_QUERY: ::grpcio::Method<super::storage::QueryRequest, super::storage::QueryResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.QueryEngine/Query",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_QUERY_ENGINE_SUBSCRIBE_CHANGE_STREAM: ::grpcio::Method<super::storage::SubscribeChangeStreamRequest, super::storage::SubscribeChangeStreamResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.QueryEngine/SubscribeChangeStream",
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

    pub fn probe_opt(&self, req: &probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<probe::ProbeResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_PROBE, req, opt)
    }

    pub fn probe(&self, req: &probe::ProbeRequest) -> ::grpcio::Result<probe::ProbeResponse> {
        self.probe_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_async_opt(&self, req: &probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<probe::ProbeResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_PROBE, req, opt)
    }

    pub fn probe_async(&self, req: &probe::ProbeRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<probe::ProbeResponse>> {
        self.probe_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn query_opt(&self, req: &super::storage::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::storage::QueryResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_QUERY, req, opt)
    }

    pub fn query(&self, req: &super::storage::QueryRequest) -> ::grpcio::Result<super::storage::QueryResponse> {
        self.query_opt(req, ::grpcio::CallOption::default())
    }

    pub fn query_async_opt(&self, req: &super::storage::QueryRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::storage::QueryResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_QUERY, req, opt)
    }

    pub fn query_async(&self, req: &super::storage::QueryRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::storage::QueryResponse>> {
        self.query_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn subscribe_change_stream_opt(&self, req: &super::storage::SubscribeChangeStreamRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::storage::SubscribeChangeStreamResponse> {
        self.client.unary_call(&METHOD_QUERY_ENGINE_SUBSCRIBE_CHANGE_STREAM, req, opt)
    }

    pub fn subscribe_change_stream(&self, req: &super::storage::SubscribeChangeStreamRequest) -> ::grpcio::Result<super::storage::SubscribeChangeStreamResponse> {
        self.subscribe_change_stream_opt(req, ::grpcio::CallOption::default())
    }

    pub fn subscribe_change_stream_async_opt(&self, req: &super::storage::SubscribeChangeStreamRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::storage::SubscribeChangeStreamResponse>> {
        self.client.unary_call_async(&METHOD_QUERY_ENGINE_SUBSCRIBE_CHANGE_STREAM, req, opt)
    }

    pub fn subscribe_change_stream_async(&self, req: &super::storage::SubscribeChangeStreamRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::storage::SubscribeChangeStreamResponse>> {
        self.subscribe_change_stream_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait QueryEngine {
    fn probe(&mut self, ctx: ::grpcio::RpcContext, _req: probe::ProbeRequest, sink: ::grpcio::UnarySink<probe::ProbeResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn query(&mut self, ctx: ::grpcio::RpcContext, _req: super::storage::QueryRequest, sink: ::grpcio::UnarySink<super::storage::QueryResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn subscribe_change_stream(&mut self, ctx: ::grpcio::RpcContext, _req: super::storage::SubscribeChangeStreamRequest, sink: ::grpcio::UnarySink<super::storage::SubscribeChangeStreamResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_query_engine<S: QueryEngine + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_PROBE, move |ctx, req, resp| {
        instance.probe(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_QUERY, move |ctx, req, resp| {
        instance.query(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_QUERY_ENGINE_SUBSCRIBE_CHANGE_STREAM, move |ctx, req, resp| {
        instance.subscribe_change_stream(ctx, req, resp)
    });
    builder.build()
}

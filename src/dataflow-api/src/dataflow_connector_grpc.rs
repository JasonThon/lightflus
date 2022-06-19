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

const METHOD_CONNECTOR_API_HANDLE_EVENT: ::grpcio::Method<super::probe::EventRequest, super::probe::EventResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/connector.ConnectorApi/HandleEvent",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_CONNECTOR_API_PROBE: ::grpcio::Method<super::probe::ProbeRequest, super::probe::ProbeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/connector.ConnectorApi/Probe",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct ConnectorApiClient {
    client: ::grpcio::Client,
}

impl ConnectorApiClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        ConnectorApiClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn handle_event_opt(&self, req: &super::probe::EventRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::probe::EventResponse> {
        self.client.unary_call(&METHOD_CONNECTOR_API_HANDLE_EVENT, req, opt)
    }

    pub fn handle_event(&self, req: &super::probe::EventRequest) -> ::grpcio::Result<super::probe::EventResponse> {
        self.handle_event_opt(req, ::grpcio::CallOption::default())
    }

    pub fn handle_event_async_opt(&self, req: &super::probe::EventRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::EventResponse>> {
        self.client.unary_call_async(&METHOD_CONNECTOR_API_HANDLE_EVENT, req, opt)
    }

    pub fn handle_event_async(&self, req: &super::probe::EventRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::EventResponse>> {
        self.handle_event_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_opt(&self, req: &super::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::probe::ProbeResponse> {
        self.client.unary_call(&METHOD_CONNECTOR_API_PROBE, req, opt)
    }

    pub fn probe(&self, req: &super::probe::ProbeRequest) -> ::grpcio::Result<super::probe::ProbeResponse> {
        self.probe_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_async_opt(&self, req: &super::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::ProbeResponse>> {
        self.client.unary_call_async(&METHOD_CONNECTOR_API_PROBE, req, opt)
    }

    pub fn probe_async(&self, req: &super::probe::ProbeRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::ProbeResponse>> {
        self.probe_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait ConnectorApi {
    fn handle_event(&mut self, ctx: ::grpcio::RpcContext, _req: super::probe::EventRequest, sink: ::grpcio::UnarySink<super::probe::EventResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn probe(&mut self, ctx: ::grpcio::RpcContext, _req: super::probe::ProbeRequest, sink: ::grpcio::UnarySink<super::probe::ProbeResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_connector_api<S: ConnectorApi + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_CONNECTOR_API_HANDLE_EVENT, move |ctx, req, resp| {
        instance.handle_event(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_CONNECTOR_API_PROBE, move |ctx, req, resp| {
        instance.probe(ctx, req, resp)
    });
    builder.build()
}

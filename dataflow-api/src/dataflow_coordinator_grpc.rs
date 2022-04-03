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

const METHOD_COORDINATOR_API_HANDLE_EVENT: ::grpcio::Method<super::dataflow_coordinator::EventRequest, super::dataflow_coordinator::EventResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/HandleEvent",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_COORDINATOR_API_PROBE: ::grpcio::Method<super::probe::ProbeRequest, super::probe::ProbeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/Probe",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct CoordinatorApiClient {
    client: ::grpcio::Client,
}

impl CoordinatorApiClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        CoordinatorApiClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn handle_event_opt(&self, req: &super::dataflow_coordinator::EventRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::dataflow_coordinator::EventResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_HANDLE_EVENT, req, opt)
    }

    pub fn handle_event(&self, req: &super::dataflow_coordinator::EventRequest) -> ::grpcio::Result<super::dataflow_coordinator::EventResponse> {
        self.handle_event_opt(req, ::grpcio::CallOption::default())
    }

    pub fn handle_event_async_opt(&self, req: &super::dataflow_coordinator::EventRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::dataflow_coordinator::EventResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_HANDLE_EVENT, req, opt)
    }

    pub fn handle_event_async(&self, req: &super::dataflow_coordinator::EventRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::dataflow_coordinator::EventResponse>> {
        self.handle_event_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_opt(&self, req: &super::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::probe::ProbeResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_PROBE, req, opt)
    }

    pub fn probe(&self, req: &super::probe::ProbeRequest) -> ::grpcio::Result<super::probe::ProbeResponse> {
        self.probe_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_async_opt(&self, req: &super::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::ProbeResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_PROBE, req, opt)
    }

    pub fn probe_async(&self, req: &super::probe::ProbeRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::probe::ProbeResponse>> {
        self.probe_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait CoordinatorApi {
    fn handle_event(&mut self, ctx: ::grpcio::RpcContext, _req: super::dataflow_coordinator::EventRequest, sink: ::grpcio::UnarySink<super::dataflow_coordinator::EventResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn probe(&mut self, ctx: ::grpcio::RpcContext, _req: super::probe::ProbeRequest, sink: ::grpcio::UnarySink<super::probe::ProbeResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_coordinator_api<S: CoordinatorApi + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_HANDLE_EVENT, move |ctx, req, resp| {
        instance.handle_event(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_PROBE, move |ctx, req, resp| {
        instance.probe(ctx, req, resp)
    });
    builder.build()
}

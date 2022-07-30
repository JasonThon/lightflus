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

const METHOD_COORDINATOR_API_PROBE: ::grpcio::Method<common::proto::probe::ProbeRequest, common::proto::probe::ProbeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/Probe",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_COORDINATOR_API_CREATE_DATAFLOW: ::grpcio::Method<super::stream::Dataflow, super::coordinator::CreateStreamGraphResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/CreateDataflow",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_COORDINATOR_API_TERMINATE_DATAFLOW: ::grpcio::Method<super::coordinator::TerminateDataflowRequest, super::coordinator::TerminateDataflowResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/TerminateDataflow",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_COORDINATOR_API_GET_DATAFLOW: ::grpcio::Method<super::coordinator::GetDataflowRequest, super::coordinator::GetDataflowResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/coordinator.CoordinatorApi/GetDataflow",
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

    pub fn probe_opt(&self, req: &common::proto::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<common::proto::probe::ProbeResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_PROBE, req, opt)
    }

    pub fn probe(&self, req: &common::proto::probe::ProbeRequest) -> ::grpcio::Result<common::proto::probe::ProbeResponse> {
        self.probe_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_async_opt(&self, req: &common::proto::probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::proto::probe::ProbeResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_PROBE, req, opt)
    }

    pub fn probe_async(&self, req: &common::proto::probe::ProbeRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<common::proto::probe::ProbeResponse>> {
        self.probe_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_dataflow_opt(&self, req: &super::stream::Dataflow, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::coordinator::CreateStreamGraphResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_CREATE_DATAFLOW, req, opt)
    }

    pub fn create_dataflow(&self, req: &super::stream::Dataflow) -> ::grpcio::Result<super::coordinator::CreateStreamGraphResponse> {
        self.create_dataflow_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_dataflow_async_opt(&self, req: &super::stream::Dataflow, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::CreateStreamGraphResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_CREATE_DATAFLOW, req, opt)
    }

    pub fn create_dataflow_async(&self, req: &super::stream::Dataflow) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::CreateStreamGraphResponse>> {
        self.create_dataflow_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn terminate_dataflow_opt(&self, req: &super::coordinator::TerminateDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::coordinator::TerminateDataflowResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_TERMINATE_DATAFLOW, req, opt)
    }

    pub fn terminate_dataflow(&self, req: &super::coordinator::TerminateDataflowRequest) -> ::grpcio::Result<super::coordinator::TerminateDataflowResponse> {
        self.terminate_dataflow_opt(req, ::grpcio::CallOption::default())
    }

    pub fn terminate_dataflow_async_opt(&self, req: &super::coordinator::TerminateDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::TerminateDataflowResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_TERMINATE_DATAFLOW, req, opt)
    }

    pub fn terminate_dataflow_async(&self, req: &super::coordinator::TerminateDataflowRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::TerminateDataflowResponse>> {
        self.terminate_dataflow_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_dataflow_opt(&self, req: &super::coordinator::GetDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::coordinator::GetDataflowResponse> {
        self.client.unary_call(&METHOD_COORDINATOR_API_GET_DATAFLOW, req, opt)
    }

    pub fn get_dataflow(&self, req: &super::coordinator::GetDataflowRequest) -> ::grpcio::Result<super::coordinator::GetDataflowResponse> {
        self.get_dataflow_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_dataflow_async_opt(&self, req: &super::coordinator::GetDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::GetDataflowResponse>> {
        self.client.unary_call_async(&METHOD_COORDINATOR_API_GET_DATAFLOW, req, opt)
    }

    pub fn get_dataflow_async(&self, req: &super::coordinator::GetDataflowRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::coordinator::GetDataflowResponse>> {
        self.get_dataflow_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait CoordinatorApi {
    fn probe(&mut self, ctx: ::grpcio::RpcContext, _req: common::proto::probe::ProbeRequest, sink: ::grpcio::UnarySink<common::proto::probe::ProbeResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn create_dataflow(&mut self, ctx: ::grpcio::RpcContext, _req: super::stream::Dataflow, sink: ::grpcio::UnarySink<super::coordinator::CreateStreamGraphResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn terminate_dataflow(&mut self, ctx: ::grpcio::RpcContext, _req: super::coordinator::TerminateDataflowRequest, sink: ::grpcio::UnarySink<super::coordinator::TerminateDataflowResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn get_dataflow(&mut self, ctx: ::grpcio::RpcContext, _req: super::coordinator::GetDataflowRequest, sink: ::grpcio::UnarySink<super::coordinator::GetDataflowResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_coordinator_api<S: CoordinatorApi + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_PROBE, move |ctx, req, resp| {
        instance.probe(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_CREATE_DATAFLOW, move |ctx, req, resp| {
        instance.create_dataflow(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_TERMINATE_DATAFLOW, move |ctx, req, resp| {
        instance.terminate_dataflow(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_COORDINATOR_API_GET_DATAFLOW, move |ctx, req, resp| {
        instance.get_dataflow(ctx, req, resp)
    });
    builder.build()
}

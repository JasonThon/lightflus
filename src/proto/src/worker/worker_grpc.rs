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

use crate::common::probe;

const METHOD_TASK_WORKER_API_PROBE: ::grpcio::Method<probe::ProbeRequest, probe::ProbeResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.TaskWorkerApi/Probe",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_TASK_WORKER_API_DISPATCH_DATA_EVENTS: ::grpcio::Method<super::worker::DispatchDataEventsRequest, super::worker::DispatchDataEventsResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.TaskWorkerApi/DispatchDataEvents",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_TASK_WORKER_API_STOP_DATAFLOW: ::grpcio::Method<super::worker::StopDataflowRequest, super::worker::StopDataflowResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.TaskWorkerApi/StopDataflow",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_TASK_WORKER_API_CREATE_DATAFLOW: ::grpcio::Method<super::worker::CreateDataflowRequest, super::worker::CreateDataflowResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/proto.TaskWorkerApi/CreateDataflow",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct TaskWorkerApiClient {
    client: ::grpcio::Client,
}

impl TaskWorkerApiClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        TaskWorkerApiClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn probe_opt(&self, req: &probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<probe::ProbeResponse> {
        self.client.unary_call(&METHOD_TASK_WORKER_API_PROBE, req, opt)
    }

    pub fn probe(&self, req: &probe::ProbeRequest) -> ::grpcio::Result<probe::ProbeResponse> {
        self.probe_opt(req, ::grpcio::CallOption::default())
    }

    pub fn probe_async_opt(&self, req: &probe::ProbeRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<probe::ProbeResponse>> {
        self.client.unary_call_async(&METHOD_TASK_WORKER_API_PROBE, req, opt)
    }

    pub fn probe_async(&self, req: &probe::ProbeRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<probe::ProbeResponse>> {
        self.probe_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn dispatch_data_events_opt(&self, req: &super::worker::DispatchDataEventsRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::worker::DispatchDataEventsResponse> {
        self.client.unary_call(&METHOD_TASK_WORKER_API_DISPATCH_DATA_EVENTS, req, opt)
    }

    pub fn dispatch_data_events(&self, req: &super::worker::DispatchDataEventsRequest) -> ::grpcio::Result<super::worker::DispatchDataEventsResponse> {
        self.dispatch_data_events_opt(req, ::grpcio::CallOption::default())
    }

    pub fn dispatch_data_events_async_opt(&self, req: &super::worker::DispatchDataEventsRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::DispatchDataEventsResponse>> {
        self.client.unary_call_async(&METHOD_TASK_WORKER_API_DISPATCH_DATA_EVENTS, req, opt)
    }

    pub fn dispatch_data_events_async(&self, req: &super::worker::DispatchDataEventsRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::DispatchDataEventsResponse>> {
        self.dispatch_data_events_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn stop_dataflow_opt(&self, req: &super::worker::StopDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::worker::StopDataflowResponse> {
        self.client.unary_call(&METHOD_TASK_WORKER_API_STOP_DATAFLOW, req, opt)
    }

    pub fn stop_dataflow(&self, req: &super::worker::StopDataflowRequest) -> ::grpcio::Result<super::worker::StopDataflowResponse> {
        self.stop_dataflow_opt(req, ::grpcio::CallOption::default())
    }

    pub fn stop_dataflow_async_opt(&self, req: &super::worker::StopDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::StopDataflowResponse>> {
        self.client.unary_call_async(&METHOD_TASK_WORKER_API_STOP_DATAFLOW, req, opt)
    }

    pub fn stop_dataflow_async(&self, req: &super::worker::StopDataflowRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::StopDataflowResponse>> {
        self.stop_dataflow_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_dataflow_opt(&self, req: &super::worker::CreateDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::worker::CreateDataflowResponse> {
        self.client.unary_call(&METHOD_TASK_WORKER_API_CREATE_DATAFLOW, req, opt)
    }

    pub fn create_dataflow(&self, req: &super::worker::CreateDataflowRequest) -> ::grpcio::Result<super::worker::CreateDataflowResponse> {
        self.create_dataflow_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_dataflow_async_opt(&self, req: &super::worker::CreateDataflowRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::CreateDataflowResponse>> {
        self.client.unary_call_async(&METHOD_TASK_WORKER_API_CREATE_DATAFLOW, req, opt)
    }

    pub fn create_dataflow_async(&self, req: &super::worker::CreateDataflowRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::worker::CreateDataflowResponse>> {
        self.create_dataflow_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::std::future::Future<Output = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait TaskWorkerApi {
    fn probe(&mut self, ctx: ::grpcio::RpcContext, _req: probe::ProbeRequest, sink: ::grpcio::UnarySink<probe::ProbeResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn dispatch_data_events(&mut self, ctx: ::grpcio::RpcContext, _req: super::worker::DispatchDataEventsRequest, sink: ::grpcio::UnarySink<super::worker::DispatchDataEventsResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn stop_dataflow(&mut self, ctx: ::grpcio::RpcContext, _req: super::worker::StopDataflowRequest, sink: ::grpcio::UnarySink<super::worker::StopDataflowResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
    fn create_dataflow(&mut self, ctx: ::grpcio::RpcContext, _req: super::worker::CreateDataflowRequest, sink: ::grpcio::UnarySink<super::worker::CreateDataflowResponse>) {
        grpcio::unimplemented_call!(ctx, sink)
    }
}

pub fn create_task_worker_api<S: TaskWorkerApi + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_TASK_WORKER_API_PROBE, move |ctx, req, resp| {
        instance.probe(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_TASK_WORKER_API_DISPATCH_DATA_EVENTS, move |ctx, req, resp| {
        instance.dispatch_data_events(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_TASK_WORKER_API_STOP_DATAFLOW, move |ctx, req, resp| {
        instance.stop_dataflow(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_TASK_WORKER_API_CREATE_DATAFLOW, move |ctx, req, resp| {
        instance.create_dataflow(ctx, req, resp)
    });
    builder.build()
}

use tokio::sync::mpsc;

use common::event;
use dataflow_api::{dataflow_connector_grpc, probe};

#[derive(Clone)]
pub(crate) struct ConnectorApiImpl {
    senders: Vec<mpsc::UnboundedSender<Vec<event::BinderEvent>>>,
}

impl ConnectorApiImpl {
    pub(crate) fn new(senders: Vec<mpsc::UnboundedSender<Vec<event::BinderEvent>>>) -> Self {
        Self { senders }
    }
}

unsafe impl Send for ConnectorApiImpl {}

unsafe impl Sync for ConnectorApiImpl {}

impl dataflow_connector_grpc::ConnectorApi for ConnectorApiImpl {
    fn handle_event(&mut self,
                    _ctx: ::grpcio::RpcContext,
                    req: probe::EventRequest,
                    sink: ::grpcio::UnarySink<probe::EventResponse>) {
        match serde_json::from_slice::<Vec<event::BinderEvent>>(req.get_data()) {
            Ok(events) => {
                common::lists::for_each(
                    &self.senders,
                    |sender| {
                        sender.send(events.clone());
                    },
                )
            }
            Err(err) => {
                sink.fail(
                    grpcio::RpcStatus::with_message(
                        grpcio::RpcStatusCode::INVALID_ARGUMENT,
                        err.to_string(),
                    )
                );
            }
        }
    }

    fn probe(&mut self,
             _ctx: grpcio::RpcContext,
             _req: probe::ProbeRequest,
             sink: grpcio::UnarySink<probe::ProbeResponse>) {
        let mut response = probe::ProbeResponse::new();
        response.set_available(true);
        sink.success(response);
    }
}
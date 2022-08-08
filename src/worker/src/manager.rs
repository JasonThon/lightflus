use std::{collections, sync};
use std::collections::HashMap;

use tokio::sync::mpsc;

use common::{err, event, types};
use common::err::ExecutionException;
use common::net::ClientConfig;
use std::ops::Deref;
use std::time;

use actix::Actor;
use serde::ser::SerializeStruct;
use tokio::task::JoinHandle;
use common::event::{LocalEvent, RowDataEvent};

use crate::constants;
use common::types::{DataEventType, SinkId};
use proto::common::common as proto_common;
use proto::common::event::DataEvent;
use proto::common::stream as proto_stream;
use proto::common::stream::Dataflow;
use proto::worker::worker::DispatchDataEventStatusEnum;
use stream::actor::{DataflowContext, Sink, SinkableMessageImpl, StreamConfig};
use stream::err::SinkException;

#[derive(Debug)]
pub struct LocalExecutorManager {
    pub job_id: proto_common::JobId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<Box<dyn Sink>>,

}

impl LocalExecutorManager {
    pub(crate) fn dispatch_events(&self, events: &Vec<DataEvent>) -> DispatchDataEventStatusEnum {
        // only one sink will be dispatched
        let sink_id_opt = events
            .iter()
            .next()
            .map(|e| e.to as SinkId);
        let local_events = events
            .iter()
            .map(|e| RowDataEvent::from(e));

        sink_id_opt
            .map(|sink_id| self.inner_sinks
                .iter()
                .filter(|sink| sink.sink_id() == sink_id)
                .next()
                .map(|sink| sink.sink(
                    SinkableMessageImpl::LocalMessage(
                        LocalEvent::RowChangeStream(local_events.collect())
                    )
                ))
                .map(|result| match result {
                    Ok(_) => DispatchDataEventStatusEnum::DONE,
                    Err(err) => {
                        log::error!("dispatch event failed: {:?}", err);
                        DispatchDataEventStatusEnum::FAILURE
                    }
                })
                .unwrap_or(DispatchDataEventStatusEnum::DONE)
            )
            .unwrap_or(DispatchDataEventStatusEnum::DONE)
    }

    pub fn new(ctx: DataflowContext) -> Self {
        let executors = ctx.create_executors();

        Self {
            job_id: ctx.job_id.clone(),
            inner_sinks: executors
                .iter()
                .map(|exec| exec.as_sinkable())
                .collect(),
            handlers: executors
                .iter()
                .map(|exec| exec.run())
                .collect(),
        }
    }

    pub fn stop(&self) -> Result<(), err::ExecutionException> {
        for sink in self.inner_sinks {
            match sink.sink(SinkableMessageImpl::LocalMessage(
                event::LocalEvent::Terminate {
                    job_id: self.job_id.clone(),
                    to: sink.sink_id(),
                }
            )) {
                Err(err) => return Err(ExecutionException::fail_send_event_to_job_graph(&self.job_id)),
                _ => {}
            }
        }

        self.handlers
            .iter()
            .for_each(|handler| handler.abort());

        Ok(())
    }
}
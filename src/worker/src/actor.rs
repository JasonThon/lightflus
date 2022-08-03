use std::{collections, sync};

use tokio::sync::mpsc;

use common::{err, event, types};
use common::err::ExecutionException;
use common::net::ClientConfig;
use std::ops::Deref;
use std::time;

use actix::Actor;
use serde::ser::SerializeStruct;
use tokio::task::JoinHandle;

use crate::constants;
use common::types::DataEventType;
use proto::common::common as proto_common;
use proto::common::stream as proto_stream;
use stream::dataflow::{DataflowContext, Sink, SinkableMessageImpl, StreamConfig};

#[derive(Debug)]
pub struct LocalExecutorManager {
    pub job_id: proto_common::JobId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<Box<dyn Sink>>,

}

impl LocalExecutorManager {
    pub fn new(ctx: DataflowContext) -> Self {
        let executors = ctx.create_executors();

        Self {
            job_id: ctx.job_id.clone(),
            inner_sinks: executors
                .iter()
                .map(|exec| exec.as_sinkable())
                .collect::<Vec<Box<dyn Sink>>>(),
            handlers: executors
                .iter()
                .map(|exec| exec.run())
                .collect::<Vec<JoinHandle<()>>>(),
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
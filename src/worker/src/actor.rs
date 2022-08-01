use std::{collections, sync};

use tokio::sync::mpsc;

use common::{err, event, types};
use common::err::ExecutionException;
use common::net::ClientConfig;
use stream::dataflow::{EventReceiver, EventSender, Executor, LocalExecutor, StreamConfig};
use std::ops::Deref;
use std::task::ready;
use std::time;

use actix::Actor;
use serde::ser::SerializeStruct;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::constants;
use stream::{dataflow as datastream, dataflow};
use stream::{pipeline, state, trigger, window};
use common::types::{DataEventType, stream};
use crate::sink::InternalSink;
use proto::common::common as proto_common;
use proto::common::stream as proto_stream;

#[derive(Debug)]
pub struct LocalExecutorManager {
    pub job_id: proto_common::JobId,
    local_executor: Vec<LocalExecutor>,

    pub config: StreamConfig,
}

impl LocalExecutorManager {
    pub fn new(ctx: dataflow::DataflowContext) -> Self {
        let executors = ctx.create_executors();
        executors
            .iter()
            .map(|exec| exec)

        Self {
            job_id: ctx.job_id.clone(),
            config: ctx.config.clone(),
        }
    }

    pub fn stop(&self) -> Result<(), err::ExecutionException> {
        for entry in &self.router {
            match exec.try_execute(event::DataEvent {
                job_id: self.job_id.into(),
                row_idx: 0,
                to: entry.0.clone(),
                event_type: DataEventType::STOP,
                data: vec![],
                old_data: vec![],
                event_time: time::SystemTime::now(),
                from: 0,
            }) {
                Err(err) => {
                    log::error!("send event failed: {:?}", &err);
                    return Err(err::ExecutionException::fail_send_event_to_job_graph(&self.job_id));
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn try_recv_data_events(&mut self, to: u64, events: Vec<event::DataEvent>) -> Result<(), err::ExecutionException> {
        match self.router.get(&to) {
            Some(exec) => match exec.try_recv_events(events) {
                Ok(_) => Ok(()),
                Err(err) =>
                    Err(err::ExecutionException {
                        kind: err::ErrorKind::SendGraphEventFailed,
                        msg: err.to_string(),
                    })
            },
            None => Ok(())
        }
    }
}
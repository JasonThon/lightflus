use std::thread::JoinHandle;

use common::err::ExecutionException;
use common::event::{LocalEvent, RowDataEvent};
use common::types::SinkId;
use proto::common::common::JobId;
use proto::common::event::{DataEvent, DataEventTypeEnum};
use proto::worker::worker::DispatchDataEventStatusEnum;
use stream::actor::{DataflowContext, Sink, SinkImpl, SinkableMessageImpl};

pub struct LocalExecutorManager {
    pub job_id: JobId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<SinkImpl>,
}

impl LocalExecutorManager {
    pub fn dispatch_events(&self, events: &Vec<DataEvent>) -> DispatchDataEventStatusEnum {
        // only one sink will be dispatched
        let local_events = events.iter().map(|e| match e.get_event_type() {
            DataEventTypeEnum::STOP => LocalEvent::Terminate {
                job_id: e.get_job_id().clone(),
                to: e.get_to_operator_id(),
            },
            _ => LocalEvent::RowChangeStream(RowDataEvent::from(e)),
        });

        events
            .iter()
            .next()
            .map(|e| e.to_operator_id as SinkId)
            .map(|sink_id| {
                self.inner_sinks
                    .iter()
                    .filter(|sink| sink.sink_id() == sink_id)
                    .next()
                    .map(|sink| {
                        local_events
                            .map(|e| sink.sink(SinkableMessageImpl::LocalMessage(e)))
                            .map(|result| match result {
                                Ok(status) => status,
                                Err(err) => {
                                    // TODO fault tolerant
                                    log::error!("dispatch event failed: {:?}", err);
                                    DispatchDataEventStatusEnum::FAILURE
                                }
                            })
                            .reduce(|accum, result| match accum {
                                DispatchDataEventStatusEnum::FAILURE => accum,
                                _ => result,
                            })
                            .unwrap_or(DispatchDataEventStatusEnum::DONE)
                    })
                    .unwrap_or(DispatchDataEventStatusEnum::DONE)
            })
            .unwrap_or(DispatchDataEventStatusEnum::DONE)
    }

    pub fn new(ctx: DataflowContext) -> Result<Self, ExecutionException> {
        if !ctx.validate() {
            return Err(ExecutionException::invalid_dataflow(&ctx.job_id));
        }

        let executors = ctx.create_executors();

        Ok(Self {
            job_id: ctx.job_id.clone(),
            inner_sinks: executors.iter().map(|exec| exec.as_sinkable()).collect(),
            handlers: executors.iter().map(|exec| exec.clone().run()).collect(),
        })
    }

    pub fn stop(&self) -> Result<(), ExecutionException> {
        for sink in &self.inner_sinks {
            let event = LocalEvent::Terminate {
                job_id: self.job_id.clone(),
                to: sink.sink_id(),
            };
            match sink.sink(SinkableMessageImpl::LocalMessage(event.clone())) {
                Err(err) => {
                    return Err(ExecutionException::sink_local_event_failure(
                        &self.job_id,
                        &event,
                        sink.sink_id(),
                        format!("{err:?}"),
                    ))
                }
                _ => {}
            }
        }

        Ok(())
    }
}

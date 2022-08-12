use tokio::task::JoinHandle;

use common::err::ExecutionException;
use common::event::{LocalEvent, RowDataEvent};
use common::types::SinkId;
use proto::common::common::JobId;
use proto::common::event::DataEvent;
use proto::worker::worker::DispatchDataEventStatusEnum;
use stream::actor::{DataflowContext, Executor, Sink, SinkImpl, SinkableMessageImpl};

pub struct LocalExecutorManager {
    pub job_id: JobId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<SinkImpl>,
}

impl LocalExecutorManager {
    pub fn dispatch_events(&self, events: &Vec<DataEvent>) -> DispatchDataEventStatusEnum {
        // only one sink will be dispatched
        let sink_id_opt = events.iter().next().map(|e| e.to_operator_id as SinkId);
        let local_events = events.iter().map(|e| RowDataEvent::from(e));

        sink_id_opt
            .map(|sink_id| {
                self.inner_sinks
                    .iter()
                    .filter(|sink| sink.sink_id() == sink_id)
                    .next()
                    .map(|sink| {
                        sink.sink(SinkableMessageImpl::LocalMessage(
                            LocalEvent::RowChangeStream(local_events.collect()),
                        ))
                    })
                    .map(|result| match result {
                        Ok(_) => DispatchDataEventStatusEnum::DONE,
                        Err(err) => {
                            log::error!("dispatch event failed: {:?}", err);
                            DispatchDataEventStatusEnum::FAILURE
                        }
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

        self.handlers.iter().for_each(|handler| handler.abort());

        Ok(())
    }
}

use common::event::LocalEvent;
use common::event::SinkableMessageImpl;
use common::types::SinkId;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::worker::DispatchDataEventStatusEnum;
use rayon::prelude::*;
use stream::actor::{DataflowContext, ExecutorImpl, Sink, SinkImpl};
use tokio::task::JoinHandle;

pub trait ExecutorManager {
    fn dispatch_events(&self, events: &Vec<KeyedDataEvent>) -> DispatchDataEventStatusEnum;
    fn get_job_id(&self) -> ResourceId;
}

pub enum ExecutorManagerImpl {
    Local(LocalExecutorManager),
}

impl ExecutorManager for ExecutorManagerImpl {
    fn dispatch_events(&self, events: &Vec<KeyedDataEvent>) -> DispatchDataEventStatusEnum {
        match self {
            ExecutorManagerImpl::Local(manager) => manager.dispatch_events(events),
        }
    }

    fn get_job_id(&self) -> ResourceId {
        match self {
            ExecutorManagerImpl::Local(manager) => manager.get_job_id(),
        }
    }
}

impl ExecutorManagerImpl {
    pub fn run(&mut self) {
        match self {
            ExecutorManagerImpl::Local(manager) => manager.run(),
        }
    }
}

pub struct LocalExecutorManager {
    pub job_id: ResourceId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<SinkImpl>,
    executors: Vec<ExecutorImpl>,
}

impl Drop for LocalExecutorManager {
    fn drop(&mut self) {
        log::info!("LocalExecutorManager for Job {:?} is dropping", self.job_id);
        for sink in &self.inner_sinks {
            let event = LocalEvent::Terminate {
                job_id: self.job_id.clone(),
                to: sink.sink_id(),
            };
            match futures_executor::block_on(
                sink.sink(SinkableMessageImpl::LocalMessage(event.clone())),
            ) {
                Err(err) => {
                    log::error!(
                        "termintate node {} failed. details: {:?}",
                        sink.sink_id(),
                        err
                    );
                }
                _ => log::info!("terminate node {} success", sink.sink_id()),
            }
        }
        self.handlers.clear();
        self.inner_sinks.clear();
    }
}

impl ExecutorManager for LocalExecutorManager {
    fn dispatch_events(&self, events: &Vec<KeyedDataEvent>) -> DispatchDataEventStatusEnum {
        // only one sink will be dispatched
        let local_events = events
            .par_iter()
            .map(|e| LocalEvent::KeyedDataStreamEvent(e.clone()));

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
                            .map(|e| {
                                futures_executor::block_on(
                                    sink.sink(SinkableMessageImpl::LocalMessage(e)),
                                )
                            })
                            .map(|result| match result {
                                Ok(status) => status,
                                Err(err) => {
                                    // TODO fault tolerant
                                    log::error!("dispatch event failed: {:?}", err);
                                    DispatchDataEventStatusEnum::Failure
                                }
                            })
                            .reduce(
                                || DispatchDataEventStatusEnum::Done,
                                |accum, result| match accum {
                                    DispatchDataEventStatusEnum::Failure => accum,
                                    _ => result,
                                },
                            )
                    })
                    .unwrap_or(DispatchDataEventStatusEnum::Done)
            })
            .unwrap_or(DispatchDataEventStatusEnum::Done)
    }

    fn get_job_id(&self) -> ResourceId {
        self.job_id.clone()
    }
}

impl LocalExecutorManager {
    pub fn new(ctx: DataflowContext) -> Self {
        let executors = ctx.create_executors();

        Self {
            job_id: ctx.job_id.clone(),
            inner_sinks: executors.iter().map(|exec| exec.as_sinkable()).collect(),
            handlers: vec![],
            executors,
        }
    }

    fn run(&mut self) {
        self.handlers = self
            .executors
            .iter()
            .map(|exec| exec.clone().run())
            .collect();
        self.executors.clear();
    }
}

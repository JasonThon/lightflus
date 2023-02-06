use common::event::LocalEvent;
use common::event::SinkableMessageImpl;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::taskmanager::SendEventToOperatorStatusEnum;
use rayon::prelude::*;
use stream::actor::{DataflowContext, Sink, SinkImpl};
use stream::err::SinkException;
use tokio::task::JoinHandle;
use tonic::async_trait;

#[async_trait]
pub trait ExecutorManager {
    async fn send_event_to_operator(
        &self,
        events: &KeyedDataEvent,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException>;
    fn get_job_id(&self) -> ResourceId;
}

pub enum ExecutorManagerImpl {
    Local(LocalExecutorManager),
}

impl Drop for ExecutorManagerImpl {
    fn drop(&mut self) {
        match self {
            Self::Local(manager) => drop(manager),
        }
    }
}

#[async_trait]
impl ExecutorManager for ExecutorManagerImpl {
    async fn send_event_to_operator(
        &self,
        events: &KeyedDataEvent,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match self {
            Self::Local(manager) => manager.send_event_to_operator(events).await,
        }
    }

    fn get_job_id(&self) -> ResourceId {
        match self {
            Self::Local(manager) => manager.get_job_id(),
        }
    }
}

pub struct LocalExecutorManager {
    pub job_id: ResourceId,
    handlers: Vec<JoinHandle<()>>,
    inner_sinks: Vec<SinkImpl>,
    ctx: DataflowContext,
}

impl Drop for LocalExecutorManager {
    fn drop(&mut self) {
        tracing::info!("LocalExecutorManager for Job {:?} is dropping", self.job_id);
        for sink in &self.inner_sinks {
            let event = LocalEvent::Terminate {
                job_id: self.job_id.clone(),
                to: sink.sink_id(),
            };
            match futures_executor::block_on(
                sink.sink(SinkableMessageImpl::LocalMessage(event.clone())),
            ) {
                Err(err) => {
                    tracing::error!(
                        "termintate node {} failed. details: {:?}",
                        sink.sink_id(),
                        err
                    );
                }
                _ => tracing::info!("terminate node {} success", sink.sink_id()),
            }
        }
        self.handlers.iter().for_each(|handler| handler.abort());
        self.handlers.clear();
        self.inner_sinks.clear();
    }
}

#[async_trait]
impl ExecutorManager for LocalExecutorManager {
    async fn send_event_to_operator(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match self
            .inner_sinks
            .iter()
            .filter(|sink| sink.sink_id() == event.to_operator_id)
            .next()
        {
            Some(sink) => {
                sink.sink(SinkableMessageImpl::LocalMessage(
                    LocalEvent::KeyedDataStreamEvent(event.clone()),
                ))
                .await
            }
            None => Ok(SendEventToOperatorStatusEnum::Done),
        }
    }

    fn get_job_id(&self) -> ResourceId {
        self.job_id.clone()
    }
}

impl LocalExecutorManager {
    pub fn new(ctx: DataflowContext) -> Self {
        Self {
            job_id: ctx.job_id.clone(),
            inner_sinks: vec![],
            handlers: vec![],
            ctx,
        }
    }
}

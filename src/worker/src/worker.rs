use std::collections::BTreeMap;

use common::err::TaskWorkerError;
use common::types::ExecutorId;
use proto::common::Dataflow;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::worker::SendEventToOperatorStatusEnum;
use stream::actor::DataflowContext;

use crate::manager::{ExecutorManager, ExecutorManagerImpl, LocalExecutorManager};

type DataflowCache = tokio::sync::RwLock<BTreeMap<ResourceId, ExecutorManagerImpl>>;

pub struct TaskWorker {
    cache: DataflowCache,
}

struct TaskWorkerBuilder {}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            cache: Default::default(),
        }
    }

    pub async fn stop_dataflow(&self, job_id: &ResourceId) -> Result<(), TaskWorkerError> {
        let mut managers = self.cache.write().await;
        managers.remove(job_id);
        Ok(())
    }

    pub async fn create_dataflow(
        &self,
        job_id: &ResourceId,
        dataflow: &Dataflow,
    ) -> Result<(), TaskWorkerError> {
        let ctx = DataflowContext::new(
            job_id.clone(),
            dataflow.meta.to_vec(),
            dataflow
                .nodes
                .iter()
                .map(|entry| (*entry.0 as ExecutorId, entry.1.clone()))
                .collect(),
        );

        let manager = LocalExecutorManager::new(ctx);
        let mut managers = self.cache.write().await;

        managers.insert(job_id.clone().into(), ExecutorManagerImpl::Local(manager));
        managers
            .get_mut(job_id)
            .iter_mut()
            .for_each(|manager| manager.run());

        Ok(())
    }

    /// TODO: if dataflow has been removed, should return error.
    pub async fn send_event_to_operator(
        &self,
        event: &KeyedDataEvent,
    ) -> Result<SendEventToOperatorStatusEnum, TaskWorkerError> {
        let managers = self.cache.read().await;
        match event.get_job_id_opt_ref() {
            Some(job_id) => match managers.get(job_id) {
                Some(manager) => manager
                    .send_event_to_operator(event)
                    .await
                    .map_err(|err| err.into_task_worker_error()),
                None => Ok(SendEventToOperatorStatusEnum::Done),
            },
            None => Ok(SendEventToOperatorStatusEnum::Done),
        }
    }
}

impl TaskWorkerBuilder {
    pub(crate) fn build(&self) -> TaskWorker {
        TaskWorker::new()
    }

    pub(crate) fn new() -> Self {
        TaskWorkerBuilder {}
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TaskWorkerConfig {
    pub port: usize,
}

pub fn new_worker() -> TaskWorker {
    TaskWorkerBuilder::new().build()
}

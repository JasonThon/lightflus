use std::collections::{BTreeMap, HashMap};

use common::collections::lang;
use common::err::TaskWorkerError;
use common::types::{ExecutorId, HashedResourceId};
use proto::common::Dataflow;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::worker::DispatchDataEventStatusEnum;
use stream::actor::DataflowContext;

use crate::manager::{ExecutorManager, ExecutorManagerImpl, LocalExecutorManager};

type DataflowCache = tokio::sync::RwLock<BTreeMap<HashedResourceId, ExecutorManagerImpl>>;

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
        let ref hashable_job_id = job_id.into();
        let mut managers = self.cache.write().await;
        managers.remove(hashable_job_id);
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
            .get_mut(&job_id.into())
            .iter_mut()
            .for_each(|manager| manager.run());

        Ok(())
    }

    pub async fn dispatch_events(
        &self,
        events: &Vec<KeyedDataEvent>,
    ) -> Result<HashMap<String, DispatchDataEventStatusEnum>, TaskWorkerError> {
        let group = lang::group(events, |e| HashedResourceId::from(e.get_job_id()));
        let managers = self.cache.read().await;

        group
            .iter()
            .map(|pair| {
                Ok(managers
                    .get(pair.0)
                    .map(|manager| {
                        (
                            format!("{:?}", &manager.get_job_id()),
                            manager.dispatch_events(
                                &group
                                    .get(&manager.get_job_id().into())
                                    .map(|events| events.iter().map(|e| (*e).clone()).collect())
                                    .unwrap(),
                            ),
                        )
                    })
                    .map(|pair| HashMap::from([pair]))
                    .unwrap_or_else(|| Default::default()))
            })
            .next()
            .unwrap_or_else(|| Ok(Default::default()))
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

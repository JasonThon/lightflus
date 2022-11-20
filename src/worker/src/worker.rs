use std::collections::{BTreeMap, HashMap};
use std::sync;

use common::collections::lang;
use common::err::TaskWorkerError;
use common::types::{ExecutorId, HashedResourceId};
use proto::common::Dataflow;
use proto::common::KeyedDataEvent;
use proto::common::ResourceId;
use proto::worker::DispatchDataEventStatusEnum;
use stream::actor::DataflowContext;

use crate::manager::{ExecutorManager, ExecutorManagerImpl, LocalExecutorManager};

type DataflowCacheRef = sync::RwLock<BTreeMap<HashedResourceId, ExecutorManagerImpl>>;

pub struct TaskWorker {
    cache: DataflowCacheRef,
}

struct TaskWorkerBuilder {}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            cache: Default::default(),
        }
    }

    pub fn stop_dataflow(&self, job_id: &ResourceId) -> Result<(), TaskWorkerError> {
        let ref hashable_job_id = job_id.into();
        self.cache
            .try_write()
            .map(|mut managers| {
                managers.remove(hashable_job_id);
            })
            .map_err(|err| TaskWorkerError::ExecutionError(err.to_string()))
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

        match self.cache.try_write() {
            Ok(mut managers) => {
                managers.insert(job_id.clone().into(), ExecutorManagerImpl::Local(manager));
                managers
                    .get_mut(&job_id.into())
                    .iter_mut()
                    .for_each(|manager| manager.run());

                Ok(())
            }
            Err(err) => Err(TaskWorkerError::ExecutionError(err.to_string())),
        }
    }

    pub fn dispatch_events(
        &self,
        events: &Vec<KeyedDataEvent>,
    ) -> Result<HashMap<String, DispatchDataEventStatusEnum>, TaskWorkerError> {
        let group = lang::group(events, |e| {
            HashedResourceId::from(e.job_id.clone().unwrap())
        });

        group
            .iter()
            .map(|pair| {
                self.cache
                    .try_read()
                    .map(|managers| {
                        managers
                            .get(pair.0)
                            .map(|manager| {
                                (
                                    format!("{:?}", &manager.get_job_id()),
                                    manager.dispatch_events(
                                        &group
                                            .get(&manager.get_job_id().into())
                                            .map(|events| {
                                                events.iter().map(|e| (*e).clone()).collect()
                                            })
                                            .unwrap(),
                                    ),
                                )
                            })
                            .map(|pair| HashMap::from([pair]))
                            .unwrap_or_else(|| Default::default())
                    })
                    .map_err(|err| TaskWorkerError::ExecutionError(format!("{:?}", err)))
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

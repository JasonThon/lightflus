use std::{collections, sync};
use std::borrow::BorrowMut;
use std::collections::{BTreeMap, HashMap};

use common::{err, event, types};
use common::err::{Error, TaskWorkerError};
use common::types::HashedJobId;
use proto::common::common::JobId;
use proto::common::event::DataEvent;
use proto::common::stream::Dataflow;
use proto::worker::worker;
use stream::actor::DataflowContext;

use crate::manager;
use crate::manager::LocalExecutorManager;

type DataflowCacheRef = sync::RwLock<BTreeMap<HashedJobId, manager::LocalExecutorManager>>;

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

    pub fn stop_dataflow(&self, job_id: JobId) -> Result<(), TaskWorkerError> {
        match self.cache.try_read()
            .map(|managers| managers
                .get(&job_id.into())
                .map(|m| m.stop())
                .map(|r| r.map_err(|err| err::TaskWorkerError::from(err)))
                .unwrap_or_else(|| Ok(()))
            )
            .map_err(|err| err::TaskWorkerError::ExecutionError(err.to_string())) {
            Ok(r) => r,
            Err(err) => Err(err)
        }
    }

    pub fn create_dataflow(&self, job_id: JobId, dataflow: Dataflow) -> Result<(), TaskWorkerError> {
        Ok(())
    }

    pub fn dispatch_events(&self, events: Vec<DataEvent>)
                           -> Result<HashMap<String, worker::DispatchDataEventStatusEnum>, TaskWorkerError> {
        events.iter()
            .map(|event| collections::HashMap::from([(event.job_id.unwrap(), vec![event.clone()])]))
            .reduce(|accum, map| {
                let mut result = collections::HashMap::from(accum);
                map.iter()
                    .for_each(|entry| result
                        .iter_mut()
                        .filter(|e| entry.0 == (*e).0)
                        .next()
                        .iter_mut()
                        .for_each(|item| {
                            let ref mut elems = entry.1.clone();
                            (*item).1.append(elems)
                        })
                    );
                result
            })
            .map(|map| map
                .iter()
                .map(|pair| {
                    self.cache
                        .try_read()
                        .map(|managers|
                            managers.get(pair.0.into())
                                .map(|m| (m.job_id.to_string(), m.dispatch_events(map.get(&m.job_id).unwrap())))
                                .iter()
                                .collect()
                        )
                        .map_err(|err| TaskWorkerError::ExecutionError(format!("{:?}", err))))
                })
                .next()
                .unwrap_or_else(|| Ok(Default::default()))
            )
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
    TaskWorkerBuilder::new()
        .build()
}

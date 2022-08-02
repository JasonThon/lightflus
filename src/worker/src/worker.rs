use std::{collections, sync};

use common::{types, event, err};
use common::err::{Error, TaskWorkerError};
use crate::actor;
use proto::worker::worker;
use proto::common::common as proto_common;
use proto::common::stream as proto_stream;
use crate::actor::LocalExecutorManager;

type DataflowCacheRef = sync::RwLock<Vec<actor::LocalExecutorManager>>;

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

    pub fn stop_dataflow(&self, job_id: proto_common::JobId) -> Result<(), err::TaskWorkerError> {
        match self.cache.try_read()
            .map(|managers| managers
                .iter()
                .filter(|m| (*m).job_id == job_id)
                .next()
                .map(|m| m.stop())
                .map(|r| r.map_err(|err| err::TaskWorkerError::from(err)))
                .unwrap_or_else(|| Ok(()))
            )
            .map_err(|err| err::TaskWorkerError::ExecutionError(err.to_string())) {
            Ok(r) => r,
            Err(err) => Err(err)
        }
    }

    pub fn create_dataflow(&self, job_id: proto_common::JobId, dataflow: proto_stream::Dataflow) -> Result<(), err::TaskWorkerError> {
        Ok(())
    }

    pub fn dispatch_events(&self, events: Vec<proto::common::event::DataEvent>)
                           -> Result<collections::HashMap<String, worker::DispatchDataEventStatusEnum>, err::TaskWorkerError> {
        Ok(collections::HashMap::new())
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

#[derive(serde::Deserialize, Debug)]
pub struct TaskWorkerConfig {
    pub port: usize,
}

pub fn new_worker() -> TaskWorker {
    TaskWorkerBuilder::new()
        .build()
}

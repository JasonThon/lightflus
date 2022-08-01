use std::collections;

use common::{types, event, err};
use common::err::Error;
use crate::actor;
use proto::worker::worker;
use proto::common::common as proto_common;
use proto::common::stream as proto_stream;

pub struct TaskWorker {
    job_pool: common::collections::ConcurrentCache<proto_common::JobId, actor::LocalExecutorManager>,
    cache: super::cache::DataflowCache,
}

struct TaskWorkerBuilder {}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            job_pool: Default::default(),
            cache: super::cache::DataflowCache::new(),
        }
    }

    pub fn stop_dataflow(&self, job_id: proto_common::JobId) -> Result<(), err::TaskWorkerError> {
        match self.job_pool.get(&job_id) {
            Some(manager) => manager.stop()
                .map(|_| {
                    self.job_pool.remove(&job_id);
                })
                .map_err(|err| err.into()),
            None => Ok(())
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

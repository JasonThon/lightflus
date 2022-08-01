use common::types;
use crate::actor;

pub struct DataflowCache {
    cache: common::collections::ConcurrentCache<types::JobID, actor::LocalExecutorManager>,
}

impl DataflowCache {
    pub fn new() -> Self {
        Self {
            cache: Default::default()
        }
    }
}
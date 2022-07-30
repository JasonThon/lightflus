use common::types;
use crate::actor;

pub struct DataflowCache {
    cache: common::collections::ConcurrentCache<types::JobID, actor::Graph>,
}

impl DataflowCache {
    pub fn new() -> Self {
        Self {
            cache: Default::default()
        }
    }
}
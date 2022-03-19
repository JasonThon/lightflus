use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Index;

use core;

use crate::source;

pub struct ClusterConfig {
    workers: Vec<std::net::SocketAddr>,
    sources: Vec<source::SourceDesc>,
}

impl ClusterConfig {
    pub fn partition_key<T: core::KeyedValue<K, V>, K: Hash, V>(&self, keyed: &T) -> std::net::SocketAddr {
        let ref mut hasher = DefaultHasher::new();
        keyed.key().hash(hasher);

        self.workers[hasher.finish() as usize % self.workers.len()]
    }

    pub fn new() -> Self{
        ClusterConfig{
            workers: vec![],
            sources: vec![]
        }
    }
}


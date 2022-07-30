use std::{collections, hash, sync};
use std::hash::Hasher;
use crate::types::JobID;

const DEFAULT_CAPACITY: u32 = 75;

unsafe impl<K, V> Send for ConcurrentCache<K, V> where K: Clone + hash::Hash + PartialEq + Ord, V: Clone {}

unsafe impl<K, V> Sync for ConcurrentCache<K, V> where K: Clone + hash::Hash + PartialEq + Ord, V: Clone {}

pub struct ConcurrentCache<K, V> where K: Clone + hash::Hash + PartialEq + Ord, V: Clone {
    buckets: Vec<sync::RwLock<collections::BTreeMap<K, V>>>,
}

impl<K, V> ConcurrentCache<K, V> where K: Clone + hash::Hash + PartialEq + Ord, V: Clone {
    pub fn get(&self, key: &K) -> Option<V> {
        let ref mut hasher = collections::hash_map::DefaultHasher::new();
        key.hash(hasher);
        let idx = hasher.finish() as usize % self.buckets.len();
        match self.buckets[idx].read() {
            Ok(result) => result.get(key).map(|val| val.clone()),
            Err(_) => None
        }
    }

    pub fn put(&self, key: K, value: V) -> Option<V> {
        let ref mut hasher = collections::hash_map::DefaultHasher::new();
        key.hash(hasher);
        let idx = hasher.finish() as usize % self.buckets.len();
        match self.buckets[idx].write() {
            Ok(mut guard) => guard.insert(key, value),
            Err(_) => None
        }
    }
    pub fn remove(&self, key: &K) {
        todo!()
    }

    pub fn get_mut(&self, key: &K) -> Option<&mut V> {
        todo!()
    }
}

impl<K, V> Default for ConcurrentCache<K, V> where K: Clone + hash::Hash + PartialEq + Ord, V: Clone {
    fn default() -> Self {
        let mut buckets = vec![];
        for _ in 0..DEFAULT_CAPACITY {
            buckets.push(sync::RwLock::new(collections::BTreeMap::new()))
        }

        Self {
            buckets,
        }
    }
}
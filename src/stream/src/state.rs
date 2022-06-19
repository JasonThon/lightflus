use std::{collections, hash, marker};
use common::types;

pub trait StateManager<K: Eq + hash::Hash + Clone, V: Clone>: Send + Sync {
    fn get_state(&self, key: &K) -> Option<V>;
}

pub struct RocksStateManager<K, V>
    where K: Eq + hash::Hash + Clone + AsRef<[u8]>,
          V: Clone + types::FromBytes {
    job_id: types::JobID,
    key: marker::PhantomData<K>,
    value: marker::PhantomData<V>,
    db: rocksdb::DB,
    cache: collections::HashMap<K, V>,
}

unsafe impl<K, V> Send for RocksStateManager<K, V>
    where K: Eq + hash::Hash + Clone + AsRef<[u8]>,
          V: Clone + types::FromBytes {}

unsafe impl<K, V> Sync for RocksStateManager<K, V>
    where K: Eq + hash::Hash + Clone + AsRef<[u8]>,
          V: Clone + types::FromBytes {}


impl<K, V> StateManager<K, V> for RocksStateManager<K, V>
    where K: Eq + hash::Hash + Clone + AsRef<[u8]>,
          V: Clone + types::FromBytes {
    fn get_state(&self, key: &K) -> Option<V> {
        match self.cache.get(key)
            .map(|value| value.clone()) {
            None => self.get_persistent(key),
            Some(value) => Some(value)
        }
    }
}

impl<K, V> RocksStateManager<K, V>
    where K: Eq + hash::Hash + Clone + AsRef<[u8]>,
          V: Clone + types::FromBytes {
    pub fn new(job_id: types::JobID) -> RocksStateManager<K, V> {
        let db = rocksdb::DB::open_default(
            format!("{}/{}", &job_id.table_id, &job_id.header_id))
            .unwrap();
        RocksStateManager {
            job_id,
            key: Default::default(),
            value: Default::default(),
            db,
            cache: Default::default(),
        }
    }

    pub fn get_persistent(&self, key: &K) -> Option<V> {
        match self.db.get(key.clone()) {
            Ok(option) => {
                match option {
                    None => None,
                    Some(data) => V::from_bytes(data)
                }
            }
            Err(err) => {
                log::error!("fail to get persistent state {}", err);
                None
            }
        }
    }
}
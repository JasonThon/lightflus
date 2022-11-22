use std::{cell::RefCell, collections::BTreeMap, path::Path};

use rocksdb::DB;

const ROCKSDB: &str = "rocksdb";
const STATE_MANAGER: &str = "STATE_MANAGER";
pub(crate) const ROCKS_STATE_PATH: &str = "ROCKS_STATE_PATH";
const DEFAULT_STATE_PATH: &str = "/tmp/state";
pub trait StateManager {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8>;
    fn set_key_state(&self, key: &[u8], value: &[u8]);
}

fn new_rocksdb_state_mgt() -> RocksStateManager {
    let path = common::utils::get_env(ROCKS_STATE_PATH).unwrap_or(DEFAULT_STATE_PATH.to_string());
    RocksStateManager::new(path)
}

pub fn new_state_mgt() -> impl StateManager {
    match state_mgt_type() {
        StateMangerType::RocksDB => StateManagerEnum::RocksDb(new_rocksdb_state_mgt()),
        StateMangerType::Memory => StateManagerEnum::Memory(MemoryStateManager::new()),
    }
}

pub struct RocksStateManager {
    db: DB,
}

impl RocksStateManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            db: DB::open_default(path)
                .map_err(|err| tracing::error!("rocks db open failed: {}", err))
                .unwrap(),
        }
    }
}

impl StateManager for RocksStateManager {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8> {
        self.db
            .get(key)
            .map(|value| value.unwrap_or(vec![]))
            .map_err(|err| tracing::error!("get state failed: {}", err))
            .unwrap_or_default()
    }

    fn set_key_state(&self, key: &[u8], value: &[u8]) {
        self.db
            .put(key, value)
            .map_err(|err| tracing::error!("set key state failed: {}", err))
            .unwrap_or_default()
    }
}

pub enum StateMangerType {
    RocksDB,
    Memory,
}

pub fn state_mgt_type() -> StateMangerType {
    common::utils::get_env(STATE_MANAGER)
        .map(|mgt_type| {
            if mgt_type.as_str() == ROCKSDB {
                StateMangerType::RocksDB
            } else {
                StateMangerType::Memory
            }
        })
        .unwrap_or(StateMangerType::Memory)
}

pub enum StateManagerEnum {
    RocksDb(RocksStateManager),
    Memory(MemoryStateManager),
}

impl StateManager for StateManagerEnum {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8> {
        match self {
            StateManagerEnum::RocksDb(manager) => manager.get_keyed_state(key),
            StateManagerEnum::Memory(manager) => manager.get_keyed_state(key),
        }
    }

    fn set_key_state(&self, key: &[u8], value: &[u8]) {
        match self {
            StateManagerEnum::RocksDb(manager) => manager.set_key_state(key, value),
            StateManagerEnum::Memory(manager) => manager.set_key_state(key, value),
        }
    }
}

pub struct MemoryStateManager {
    cache: RefCell<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl StateManager for MemoryStateManager {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8> {
        self.cache
            .borrow()
            .get(&key.to_vec())
            .map(|data| data.clone())
            .unwrap_or(vec![])
    }

    fn set_key_state(&self, key: &[u8], value: &[u8]) {
        self.cache.borrow_mut().insert(key.to_vec(), value.to_vec());
    }
}

impl MemoryStateManager {
    pub fn new() -> Self {
        Self {
            cache: Default::default(),
        }
    }
}

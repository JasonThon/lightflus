use std::{cell::RefCell, collections::BTreeMap, path::Path};

use sled::Db;
use proto::common::ResourceId;

const KEY_VALUE: &str = "key_value";
const STATE_MANAGER: &str = "STATE_MANAGER";
pub(crate) const KEY_VALUE_STATE_PATH: &str = "KEY_VALUE_STATE_PATH";
const DEFAULT_STATE_PATH: &str = "/tmp/state";
pub trait StateManager {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8>;
    fn set_key_state(&self, key: &[u8], value: &[u8]);
}

fn new_key_value_state_mgt(resource_id: &ResourceId) -> KeyValueStateManager {
    let mut path =
        common::utils::get_env(KEY_VALUE_STATE_PATH).unwrap_or(DEFAULT_STATE_PATH.to_string());
    path.push_str("/");
    path.push_str(&resource_id.namespace_id);
    path.push_str(&resource_id.resource_id);
    KeyValueStateManager::new(path)
}

pub fn new_state_mgt(resource_id: &ResourceId) -> impl StateManager {
    match state_mgt_type() {
        StateMangerType::KeyValue => StateManagerEnum::KeyValue(new_key_value_state_mgt(resource_id)),
        StateMangerType::Memory => StateManagerEnum::Memory(MemoryStateManager::new()),
    }
}

pub struct KeyValueStateManager {
    db: Db,
}

impl KeyValueStateManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            db: sled::open(path)
                .map_err(|err| tracing::error!("db open failed: {}", err))
                .unwrap(),
        }
    }
}

impl StateManager for KeyValueStateManager {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8> {
        self.db
            .get(key)
            .map(|value| value.map(|v| v.to_vec()).unwrap_or(vec![]))
            .map_err(|err| tracing::error!("get state failed: {}", err))
            .unwrap_or_default()
    }

    fn set_key_state(&self, key: &[u8], value: &[u8]) {
        self.db
            .insert(key, value)
            .map(|_| {})
            .map_err(|err| tracing::error!("set key state failed: {}", err))
            .unwrap_or_default()
    }
}

pub enum StateMangerType {
    KeyValue,
    Memory,
}

pub fn state_mgt_type() -> StateMangerType {
    common::utils::get_env(STATE_MANAGER)
        .map(|mgt_type| {
            if mgt_type.as_str() == KEY_VALUE {
                StateMangerType::KeyValue
            } else {
                StateMangerType::Memory
            }
        })
        .unwrap_or(StateMangerType::Memory)
}

pub enum StateManagerEnum {
    KeyValue(KeyValueStateManager),
    Memory(MemoryStateManager),
}

impl StateManager for StateManagerEnum {
    fn get_keyed_state(&self, key: &[u8]) -> Vec<u8> {
        match self {
            StateManagerEnum::KeyValue(manager) => manager.get_keyed_state(key),
            StateManagerEnum::Memory(manager) => manager.get_keyed_state(key),
        }
    }

    fn set_key_state(&self, key: &[u8], value: &[u8]) {
        match self {
            StateManagerEnum::KeyValue(manager) => manager.set_key_state(key, value),
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

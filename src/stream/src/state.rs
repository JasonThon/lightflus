use std::path::Path;

use rocksdb::DB;

const ROCKSDB: &str = "rocksdb";
const STATE_MANAGER: &str = "STATE_MANAGER";
const ROCKS_STATE_PATH: &str = "ROCKS_STATE_PATH";
const DEFAULT_STATE_PATH: &str = "/tmp/state";
pub trait StateManager {}

fn new_rowsdb_state_mgt() -> RocksStateManager {
    let path = common::utils::get_env(ROCKS_STATE_PATH).unwrap_or(DEFAULT_STATE_PATH.to_string());
    RocksStateManager::new(path)
}

pub fn new_state_mgt() -> impl StateManager {
    match state_mgt_type() {
        StateMangerType::RocksDB => StateManagerEnum::RocksDb(new_rowsdb_state_mgt()),
    }
}

pub struct RocksStateManager {
    db: Result<DB, rocksdb::Error>,
}

impl RocksStateManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            db: DB::open_default(path),
        }
    }
}

impl StateManager for RocksStateManager {}

enum StateMangerType {
    RocksDB,
}

pub fn state_mgt_type() -> StateMangerType {
    common::utils::get_env(STATE_MANAGER)
        .map(|mgt_type| {
            if mgt_type.as_str() == ROCKSDB {
                StateMangerType::RocksDB
            } else {
                StateMangerType::RocksDB
            }
        })
        .unwrap_or(StateMangerType::RocksDB)
}

pub enum StateManagerEnum {
    RocksDb(RocksStateManager),
}

impl StateManager for StateManagerEnum {}

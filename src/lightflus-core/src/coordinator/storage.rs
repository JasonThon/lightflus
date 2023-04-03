use std::{collections::BTreeMap, fmt::Display};

use common::utils;
use prost::Message;
use proto::common::{Dataflow, ResourceId};

#[derive(serde::Deserialize, Clone, Debug)]
pub enum DataflowStorageBuilder {
    Local { dataflow_store_path: String },
    Memory,
}

impl DataflowStorageBuilder {
    pub fn build(&self) -> Box<dyn DataflowStorage> {
        match self {
            Self::Local {
                dataflow_store_path,
            } => Box::new(LocalDataflowStorage::new(dataflow_store_path)),
            Self::Memory => Box::new(MemDataflowStorage::default()),
        }
    }
}

pub trait DataflowStorage: Send + Sync {
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), StorageError>;
    fn get(&self, job_id: &ResourceId) -> Option<Dataflow>;
    fn may_exists(&self, job_id: &ResourceId) -> bool;
    fn delete(&mut self, job_id: &ResourceId) -> Result<(), StorageError>;
}

#[derive(Clone, Debug)]
pub(crate) struct LocalDataflowStorage {
    db: sled::Db,
}

impl LocalDataflowStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self {
            db: sled::open(path).expect("open sleddb failed"),
        }
    }
}

impl DataflowStorage for LocalDataflowStorage {
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), StorageError> {
        self.db
            .insert(
                dataflow
                    .job_id
                    .as_ref()
                    .map(|key| key.encode_to_vec())
                    .unwrap_or_default(),
                dataflow.encode_to_vec(),
            )
            .map(|_| {})
            .map_err(|err| StorageError::SaveDataflowFailed(err))
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self
            .db
            .get(&job_id.encode_to_vec())
            .map(|data| data.and_then(|buf| utils::from_pb_slice(&buf).ok()))
            .map_err(|err| StorageError::GetDataflowFailed(err))
        {
            Ok(result) => result,
            Err(err) => {
                tracing::error!("get dataflow {:?} failed because: {}", job_id, err);
                None
            }
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.db
            .contains_key(job_id.encode_to_vec())
            .unwrap_or(false)
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), StorageError> {
        self.db
            .remove(job_id.encode_to_vec())
            .map(|_| {})
            .map_err(|err| StorageError::DeleteDataflowFailed(err))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MemDataflowStorage {
    cache: BTreeMap<ResourceId, Dataflow>,
}

impl DataflowStorage for MemDataflowStorage {
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), StorageError> {
        self.cache.insert(dataflow.get_job_id(), dataflow.clone());
        Ok(())
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.cache.get(job_id).map(|dataflow| dataflow.clone())
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.cache.contains_key(job_id)
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), StorageError> {
        self.cache.remove(job_id);
        Ok(())
    }
}

#[derive(Debug)]
pub enum StorageError {
    SaveDataflowFailed(sled::Error),
    DeleteDataflowFailed(sled::Error),
    GetDataflowFailed(sled::Error),
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::SaveDataflowFailed(err) => {
                f.write_fmt(format_args!("save dataflow failed: {}", err))
            }
            StorageError::DeleteDataflowFailed(err) => {
                f.write_fmt(format_args!("delete dataflow failed: {}", err))
            }
            StorageError::GetDataflowFailed(err) => {
                f.write_fmt(format_args!("get dataflow failed: {}", err))
            }
        }
    }
}

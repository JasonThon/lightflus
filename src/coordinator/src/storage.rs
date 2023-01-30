use std::collections::BTreeMap;

use common::{
    err::{CommonException, ErrorKind},
    utils,
};
use prost::Message;
use proto::common::{Dataflow, ResourceId};

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) enum DataflowStorageBuilder {
    Local { dataflow_store_path: String },
    Memory,
}

impl DataflowStorageBuilder {
    pub(crate) fn build(&self) -> Box<dyn DataflowStorage> {
        match self {
            Self::Local {
                dataflow_store_path,
            } => Box::new(LocalDataflowStorage::new(dataflow_store_path)),
            Self::Memory => Box::new(MemDataflowStorage::default()),
        }
    }
}

pub(crate) trait DataflowStorage: Send + Sync {
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), CommonException>;
    fn get(&self, job_id: &ResourceId) -> Option<Dataflow>;
    fn may_exists(&self, job_id: &ResourceId) -> bool;
    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException>;
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
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), CommonException> {
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
            .map_err(|err| CommonException {
                kind: ErrorKind::SaveDataflowFailed,
                message: err.to_string(),
            })
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self
            .db
            .get(&job_id.encode_to_vec())
            .map(|data| data.and_then(|buf| utils::from_pb_slice(&buf).ok()))
            .map_err(|err| CommonException {
                kind: ErrorKind::GetDataflowFailed,
                message: err.to_string(),
            }) {
            Ok(result) => result,
            Err(err) => {
                tracing::error!("get dataflow {:?} failed because: {:?}", job_id, err);
                None
            }
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.db
            .contains_key(job_id.encode_to_vec())
            .unwrap_or(false)
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        self.db
            .remove(job_id.encode_to_vec())
            .map(|_| {})
            .map_err(|err| CommonException {
                kind: ErrorKind::DeleteDataflowFailed,
                message: err.to_string(),
            })
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MemDataflowStorage {
    cache: BTreeMap<ResourceId, Dataflow>,
}

impl DataflowStorage for MemDataflowStorage {
    fn save(&mut self, dataflow: &Dataflow) -> Result<(), CommonException> {
        self.cache.insert(dataflow.get_job_id(), dataflow.clone());
        Ok(())
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.cache.get(job_id).map(|dataflow| dataflow.clone())
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.cache.contains_key(job_id)
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        self.cache.remove(job_id);
        Ok(())
    }
}

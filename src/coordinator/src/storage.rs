use std::collections::BTreeMap;

use common::{
    err::{CommonException, ErrorKind},
    types::HashedResourceId,
    utils,
};
use prost::Message;
use proto::common::{Dataflow, ResourceId};

pub(crate) trait DataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException>;
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
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
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
    cache: BTreeMap<HashedResourceId, Dataflow>,
}

impl DataflowStorage for MemDataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        self.cache.insert(
            HashedResourceId::from(dataflow.job_id.as_ref().unwrap()),
            dataflow.clone(),
        );
        Ok(())
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.cache
            .get(&HashedResourceId::from(job_id))
            .map(|dataflow| dataflow.clone())
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.cache.contains_key(&job_id.into())
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        self.cache.remove(&job_id.into());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) enum DataflowStorageImpl {
    Local(LocalDataflowStorage),
    Memory(MemDataflowStorage),
}

impl DataflowStorageImpl {
    pub(crate) fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        match self {
            Self::Local(storage) => storage.save(dataflow),
            Self::Memory(storage) => storage.save(dataflow),
        }
    }

    pub(crate) fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self {
            Self::Local(storage) => storage.get(job_id),
            Self::Memory(storage) => storage.get(job_id),
        }
    }

    pub(crate) fn may_exists(&self, job_id: &ResourceId) -> bool {
        match self {
            Self::Local(storage) => storage.may_exists(job_id),
            Self::Memory(storage) => storage.may_exists(job_id),
        }
    }

    pub(crate) fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        match self {
            DataflowStorageImpl::Local(storage) => storage.delete(job_id),
            DataflowStorageImpl::Memory(storage) => storage.delete(job_id),
        }
    }
}

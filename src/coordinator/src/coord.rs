use std::collections::BTreeMap;
use std::sync::Arc;

use common::err::CommonException;
use common::err::ErrorKind;
use common::net::cluster;

use common::types::HashedResourceId;
use common::utils;
use common::utils::validate_dataflow;
use prost::Message;
use proto::common::Dataflow;
use proto::common::DataflowStatus;
use proto::common::ResourceId;
use rocksdb::DB;

pub(crate) trait DataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException>;
    fn get(&self, job_id: &ResourceId) -> Option<Dataflow>;
    fn may_exists(&self, job_id: &ResourceId) -> bool;
    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException>;
}

#[derive(Clone, Debug)]
pub struct RocksDataflowStorage {
    db: Arc<DB>,
}

impl DataflowStorage for RocksDataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        self.db
            .put(
                dataflow
                    .job_id
                    .as_ref()
                    .map(|key| key.encode_to_vec())
                    .unwrap_or_default(),
                dataflow.encode_to_vec(),
            )
            .map_err(|err| CommonException {
                kind: ErrorKind::SaveDataflowFailed,
                message: err.into_string(),
            })
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self
            .db
            .get(&job_id.encode_to_vec())
            .map(|data| data.and_then(|buf| utils::from_pb_slice(&buf).ok()))
            .map_err(|err| CommonException {
                kind: ErrorKind::GetDataflowFailed,
                message: err.into_string(),
            }) {
            Ok(result) => result,
            Err(err) => {
                log::error!("get dataflow {:?} failed because: {:?}", job_id, err);
                None
            }
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        self.db.key_may_exist(job_id.encode_to_vec())
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        self.db
            .delete(job_id.encode_to_vec())
            .map_err(|err| CommonException {
                kind: ErrorKind::DeleteDataflowFailed,
                message: err.into_string(),
            })
    }
}

#[derive(Clone, Debug, Default)]
pub struct MemDataflowStorage {
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
pub enum DataflowStorageImpl {
    RocksDB(RocksDataflowStorage),
    Memory(MemDataflowStorage),
}

impl DataflowStorageImpl {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        match self {
            Self::RocksDB(storage) => storage.save(dataflow),
            Self::Memory(storage) => storage.save(dataflow),
        }
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self {
            Self::RocksDB(storage) => storage.get(job_id),
            Self::Memory(storage) => storage.get(job_id),
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        match self {
            Self::RocksDB(storage) => storage.may_exists(job_id),
            Self::Memory(storage) => storage.may_exists(job_id),
        }
    }

    fn delete(&mut self, job_id: &ResourceId) -> Result<(), CommonException> {
        match self {
            DataflowStorageImpl::RocksDB(storage) => storage.delete(job_id),
            DataflowStorageImpl::Memory(storage) => storage.delete(job_id),
        }
    }
}

#[derive(Clone)]
pub struct Coordinator {
    dataflow_storage: DataflowStorageImpl,
    cluster: cluster::Cluster,
}

impl Coordinator {
    pub fn new(
        job_storage: DataflowStorageImpl,
        cluster_config: &Vec<cluster::NodeConfig>,
    ) -> Self {
        Coordinator {
            dataflow_storage: job_storage,
            cluster: cluster::Cluster::new(cluster_config),
        }
    }

    pub fn create_dataflow(&mut self, mut dataflow: Dataflow) -> Result<(), tonic::Status> {
        validate_dataflow(&dataflow)
            .map_err(|err| tonic::Status::invalid_argument(format!("{:?}", err)))
            .and_then(|_| {
                self.cluster.partition_dataflow(&mut dataflow);
                let terminate_result = self.terminate_dataflow(dataflow.job_id.as_ref().unwrap());
                if terminate_result.is_err() {
                    return terminate_result.map(|_| ());
                }

                match self.dataflow_storage.save(dataflow.clone()) {
                    Err(err) => return Err(tonic::Status::internal(err.message)),
                    _ => {}
                }

                self.cluster.create_dataflow(&dataflow)
            })
    }

    pub fn terminate_dataflow(
        &mut self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, tonic::Status> {
        if !self.dataflow_storage.may_exists(job_id) {
            Ok(DataflowStatus::Closed)
        } else {
            self.dataflow_storage
                .delete(job_id)
                .map_err(|err| {
                    log::error!("delete dataflow failed: {:?}", err);
                    tonic::Status::internal(err.message)
                })
                .and_then(|_| self.cluster.terminate_dataflow(job_id))
        }
    }

    pub fn get_dataflow(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.dataflow_storage.get(job_id)
    }

    pub fn probe_state(&mut self) {
        self.cluster.probe_state()
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub storage: DataflowStorageConfig,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub enum DataflowStorageConfig {
    RocksDB { dataflow_store_path: String },
    Memory,
}

impl DataflowStorageConfig {
    pub fn to_dataflow_storage(&self) -> DataflowStorageImpl {
        match self {
            Self::RocksDB {
                dataflow_store_path,
            } => DataflowStorageImpl::RocksDB(RocksDataflowStorage {
                db: Arc::new(DB::open_default(dataflow_store_path).expect("open rocksdb failed")),
            }),
            Self::Memory => DataflowStorageImpl::Memory(Default::default()),
        }
    }
}

pub struct CoordinatorException {}

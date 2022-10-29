use std::collections::HashMap;

use common::err::ApiError;
use common::err::CommonException;
use common::err::ErrorKind;
use common::net::cluster;
use common::net::status;

use proto::common::common::ResourceId;
use proto::common::stream::Dataflow;
use proto::common::stream::DataflowStatus;
use protobuf::Message;
use protobuf::RepeatedField;
use rocksdb::DB;

pub(crate) trait DataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException>;
    fn get(&self, job_id: &ResourceId) -> Option<Dataflow>;
    fn may_exists(&self, job_id: &ResourceId) -> bool;
    fn delete(&self, job_id: &ResourceId) -> Result<(), CommonException>;
}

#[derive(Clone, Debug)]
pub struct RocksDataflowStorage {
    dataflow_store_path: String,
}

impl DataflowStorage for RocksDataflowStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        DB::open_default(self.dataflow_store_path.as_str())
            .map_err(|err| CommonException {
                kind: ErrorKind::SaveDataflowFailed,
                message: err.into_string(),
            })
            .and_then(|db| {
                dataflow
                    .get_job_id()
                    .write_to_bytes()
                    .map_err(|err| CommonException::from(err))
                    .and_then(|job_id_bytes| {
                        dataflow
                            .write_to_bytes()
                            .map_err(|err| err.into())
                            .and_then(|buf| {
                                db.put(job_id_bytes, buf).map_err(|err| CommonException {
                                    kind: ErrorKind::SaveDataflowFailed,
                                    message: err.into_string(),
                                })
                            })
                    })
            })
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match DB::open_default(self.dataflow_store_path.as_str())
            .map_err(|err| CommonException {
                kind: ErrorKind::OpenDBFailed,
                message: err.into_string(),
            })
            .and_then(|db| {
                job_id
                    .write_to_bytes()
                    .map_err(|err| CommonException::from(err))
                    .and_then(|key| {
                        db.get(key)
                            .map(|data| data.and_then(|buf| Dataflow::parse_from_bytes(&buf).ok()))
                            .map_err(|err| CommonException {
                                kind: ErrorKind::GetDataflowFailed,
                                message: err.into_string(),
                            })
                    })
            }) {
            Ok(result) => result,
            Err(err) => {
                log::error!("get dataflow {:?} failed because: {:?}", job_id, err);
                None
            }
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        match DB::open_default(self.dataflow_store_path.as_str())
            .map_err(|err| CommonException {
                kind: ErrorKind::OpenDBFailed,
                message: err.into_string(),
            })
            .and_then(|db| {
                job_id
                    .write_to_bytes()
                    .map_err(|err| err.into())
                    .map(|key| db.key_may_exist(key))
            }) {
            Ok(exist) => exist,
            Err(err) => {
                log::error!("call dataflow exists error {:?}", err);
                false
            }
        }
    }

    fn delete(&self, job_id: &ResourceId) -> Result<(), CommonException> {
        DB::open_default(self.dataflow_store_path.as_str())
            .map_err(|err| CommonException {
                kind: ErrorKind::OpenDBFailed,
                message: err.into_string(),
            })
            .and_then(|db| {
                job_id
                    .write_to_bytes()
                    .map_err(|err| err.into())
                    .and_then(|key| {
                        db.delete(key).map_err(|err| CommonException {
                            kind: ErrorKind::DeleteDataflowFailed,
                            message: err.into_string(),
                        })
                    })
            })
    }
}

#[derive(Clone, Debug)]
pub enum DataflowStorageImpl {
    RocksDB(RocksDataflowStorage),
}

impl DataflowStorageImpl {
    fn save(
        &mut self,
        job_id: &ResourceId,
        map: &HashMap<String, Dataflow>,
    ) -> Result<(), CommonException> {
        let mut metas = vec![];
        let mut operator_infos = HashMap::new();
        map.iter().for_each(|entry| {
            entry.1.get_meta().iter().for_each(|meta| {
                if !metas.contains(meta) {
                    metas.push(meta.clone())
                }
            });
            entry.1.get_nodes().iter().for_each(|sub_entry| {
                operator_infos.insert(*sub_entry.0, sub_entry.1.clone());
            })
        });
        let mut dataflow = Dataflow::default();
        dataflow.set_job_id(job_id.clone());
        dataflow.set_meta(RepeatedField::from_vec(metas));
        dataflow.set_nodes(operator_infos);

        match self {
            Self::RocksDB(storage) => storage.save(dataflow),
            _ => Ok(()),
        }
    }

    fn get(&self, job_id: &ResourceId) -> Option<Dataflow> {
        match self {
            DataflowStorageImpl::RocksDB(storage) => storage.get(job_id),
        }
    }

    fn may_exists(&self, job_id: &ResourceId) -> bool {
        match self {
            DataflowStorageImpl::RocksDB(storage) => storage.may_exists(job_id),
        }
    }

    fn delete(&self, job_id: &ResourceId) -> Result<(), CommonException> {
        match self {
            DataflowStorageImpl::RocksDB(storage) => storage.delete(job_id),
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

    pub fn create_dataflow(&mut self, dataflow: Dataflow) -> Result<(), ApiError> {
        let job_id = dataflow.get_job_id();

        let map = self.cluster.partition_dataflow(&dataflow);
        match self.dataflow_storage.save(job_id, &map) {
            Err(err) => return err.to_api_error(),
            _ => {}
        }

        let terminate_result = self.terminate_dataflow(job_id);
        if terminate_result.is_err() {
            return terminate_result.map(|_| ());
        }

        self.cluster.create_dataflow(map)
    }

    pub fn terminate_dataflow(&self, job_id: &ResourceId) -> Result<DataflowStatus, ApiError> {
        if !self.dataflow_storage.may_exists(job_id) {
            Ok(DataflowStatus::CLOSED)
        } else {
            self.dataflow_storage
                .delete(job_id)
                .map_err(|err| {
                    log::error!("delete dataflow failed: {:?}", err);
                    ApiError {
                        code: status::SERVICE_INTERNAL_ERROR,
                        msg: err.message,
                    }
                })
                .and_then(|_| self.cluster.terminate_dataflow(job_id))
        }
    }

    pub fn get_dataflow(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.dataflow_storage.get(job_id)
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
}

impl DataflowStorageConfig {
    pub fn to_dataflow_storage(&self) -> DataflowStorageImpl {
        match self {
            DataflowStorageConfig::RocksDB {
                dataflow_store_path,
            } => DataflowStorageImpl::RocksDB(RocksDataflowStorage {
                dataflow_store_path: dataflow_store_path.clone(),
            }),
        }
    }
}

pub struct CoordinatorException {}

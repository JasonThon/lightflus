use common::err::ApiError;
use common::err::CommonException;
use common::err::ErrorKind;
use common::net::cluster;
use common::net::status;

use proto::{
    common::stream::Dataflow,
    worker::{cli, worker::CreateDataflowRequest},
};
use protobuf::Message;
use rocksdb::DB;

pub(crate) trait JobStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException>;
}

#[derive(Clone, Debug)]
pub(crate) struct RocksDBJobStorage {
    path: String,
}

impl JobStorage for RocksDBJobStorage {
    fn save(&mut self, dataflow: Dataflow) -> Result<(), CommonException> {
        DB::open_default(self.path)
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
}

#[derive(Clone, Debug)]
pub enum JobStorageImpl {
    RocksDB(RocksDBJobStorage),
}

impl JobStorageImpl {
    fn save(
        &mut self,
        job_id: &proto::common::common::JobId,
        map: &std::collections::HashMap<String, Dataflow>,
    ) -> Result<(), CommonException> {
        let mut metas = vec![];
        let mut operator_infos = std::collections::HashMap::new();
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
        dataflow.set_meta(metas);
        dataflow.set_nodes(operator_infos);

        match self {
            Self::RocksDB(storage) => storage.save(dataflow),
            _ => Ok(()),
        }
    }
}

#[derive(Clone)]
pub struct Coordinator {
    job_storage: JobStorageImpl,
    cluster: cluster::Cluster,
}

impl Coordinator {
    pub fn new(job_storage: JobStorageImpl, cluster_config: &Vec<cluster::NodeConfig>) -> Self {
        Coordinator {
            job_storage,
            cluster: cluster::Cluster::new(cluster_config),
        }
    }

    pub fn create_dataflow(&mut self, dataflow: Dataflow) -> Result<(), ApiError> {
        let job_id = dataflow.get_job_id();
        let map = self.cluster.partition_dataflow(dataflow);
        match self.job_storage.save(job_id, &map) {
            Err(err) => return err.to_api_error(),
            _ => {}
        }

        for elem in map {
            let client = cli::new_dataflow_worker_client(cli::DataflowWorkerConfig {
                host: None,
                port: None,
                uri: Some(elem.0.clone()),
            });
            let ref mut req = CreateDataflowRequest::new();
            req.set_job_id(elem.1.get_job_id().clone());
            req.set_dataflow(elem.1.clone());
            match client
                .create_dataflow(req)
                .map_err(|err| ApiError::from(err))
                .and_then(|resp| {
                    if resp.get_resp().get_status() == status::SUCCESS {
                        Ok(())
                    } else {
                        Err(ApiError::from(resp.get_resp()))
                    }
                }) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
}

pub struct CoordinatorException {}

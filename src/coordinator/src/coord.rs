use common::net::cluster;

use proto::common::Dataflow;
use proto::common::DataflowStatus;
use proto::common::ResourceId;

use crate::managers::Dispatcher;
use crate::storage::DataflowStorageImpl;
use crate::storage::PersistDataflowStorage;

pub struct Coordinator {
    dispatcher: Dispatcher,
}

impl Coordinator {
    pub fn new(config: &CoordinatorConfig) -> Self {
        let dispatcher = Dispatcher::new(config);
        Coordinator { dispatcher }
    }

    pub async fn create_dataflow(&mut self, mut dataflow: Dataflow) -> Result<(), tonic::Status> {
        match dataflow
            .validate()
            .map_err(|err| tonic::Status::invalid_argument(format!("{:?}", err)))
        {
            Ok(_) => {
                let terminate_result = self
                    .terminate_dataflow(dataflow.job_id.as_ref().unwrap())
                    .await;
                if terminate_result.is_err() {
                    return terminate_result.map(|_| ());
                }
                self.dispatcher
                    .create_dataflow(&mut dataflow)
                    .await
                    .map_err(|err| err.to_tonic_status())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn terminate_dataflow(
        &mut self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, tonic::Status> {
        self.dispatcher
            .terminate_dataflow(job_id)
            .await
            .map_err(|err| err.to_tonic_status())
    }

    pub fn get_dataflow(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.dispatcher.get_dataflow(job_id)
    }

    pub async fn probe_state(&mut self) {
        self.dispatcher.probe_cluster_state().await
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
    Persist { dataflow_store_path: String },
    Memory,
}

impl DataflowStorageConfig {
    pub fn to_dataflow_storage(&self) -> DataflowStorageImpl {
        match self {
            Self::Persist {
                dataflow_store_path,
            } => DataflowStorageImpl::Persist(PersistDataflowStorage::new(dataflow_store_path)),
            Self::Memory => DataflowStorageImpl::Memory(Default::default()),
        }
    }
}

pub struct CoordinatorException {}

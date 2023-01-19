use common::net::{cluster, AckResponderBuilder, HeartbeatConfig};

use crate::storage::{DataflowStorageImpl, PersistDataflowStorage};

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub storage: DataflowStorageConfig,
    pub heartbeat: HeartbeatConfig,
    pub ack: AckResponderBuilder,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) enum DataflowStorageConfig {
    Persist { dataflow_store_path: String },
    Memory,
}

impl DataflowStorageConfig {
    pub(crate) fn to_dataflow_storage(&self) -> DataflowStorageImpl {
        match self {
            Self::Persist {
                dataflow_store_path,
            } => DataflowStorageImpl::Persist(PersistDataflowStorage::new(dataflow_store_path)),
            Self::Memory => DataflowStorageImpl::Memory(Default::default()),
        }
    }
}

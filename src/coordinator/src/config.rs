use common::net::{cluster, AckResponderBuilder, HeartbeatBuilder};

use crate::storage::{DataflowStorageImpl, LocalDataflowStorage};

#[derive(serde::Deserialize, Clone, Debug)]
pub(crate) struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub storage: DataflowStorageConfig,
    pub heartbeat: HeartbeatBuilder,
    pub ack: AckResponderBuilder,
    pub rpc_timeout: u64,
    pub connect_timeout: u64,
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
            } => DataflowStorageImpl::Local(LocalDataflowStorage::new(dataflow_store_path)),
            Self::Memory => DataflowStorageImpl::Memory(Default::default()),
        }
    }
}

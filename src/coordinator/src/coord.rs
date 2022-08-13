use std::{collections::HashMap, sync};

use crate::cluster;
use common::err::ApiError;
use proto::common::stream::Dataflow;

pub const COORD_JOB_GRAPH_COLLECTION: &str = "coord.job.graph";

pub enum JobStorage {
    PgSQL,
}

impl JobStorage {}

pub struct Coordinator {
    job_storage: JobStorage,
    cluster: cluster::Cluster,
}

impl Coordinator {
    pub fn new(job_storage: JobStorage, cluster_config: &Vec<cluster::NodeConfig>) -> Self {
        Coordinator {
            job_storage,
            cluster: cluster::Cluster::new(cluster_config),
        }
    }

    pub fn create_dataflow(&mut self, dataflow: Dataflow) -> Result<(), ApiError> {
        todo!()
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
}

pub struct CoordinatorException {}

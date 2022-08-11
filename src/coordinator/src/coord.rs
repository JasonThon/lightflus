use std::{collections, sync};

use tokio::sync::mpsc;

use crate::cluster;
use common::{err, event, types};

pub const COORD_JOB_GRAPH_COLLECTION: &str = "coord.job.graph";

pub enum JobStorage {
    PgSQL
}

impl JobStorage {}

pub struct Coordinator {
    job_storage: JobStorage,
    connector_proxy: String,
}

impl Coordinator {
    pub fn new(job_storage: JobStorage,
               connector_proxy: String) -> Self {
        Coordinator {
            job_storage,
            connector_proxy,
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub conn_proxy: String,
}
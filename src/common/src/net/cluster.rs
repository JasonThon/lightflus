use crate::collections::lang;
use crate::err::ApiError;
use crate::net::{to_host_addr, PersistableHostAddr};
use crate::types;
use crate::types::SingleKV;
use crate::utils;
use proto::common::common::ResourceId;
use proto::common::probe;
use proto::common::stream::{Dataflow, DataflowStatus};
use proto::worker::worker::{CreateDataflowRequest, StopDataflowRequest};
use proto::worker::{self, cli};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::status;

#[derive(Clone, Eq, PartialEq, Debug)]
enum NodeStatus {
    Pending,
    Running,
    Unreachable,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct Node {
    status: NodeStatus,
    pub host_addr: PersistableHostAddr,
}

impl Node {
    pub(crate) fn probe_state(&mut self) {
        let client = worker::cli::new_dataflow_worker_client(worker::cli::DataflowWorkerConfig {
            host: None,
            port: None,
            uri: Some(self.host_addr.as_uri()),
        });

        let ref mut request = probe::ProbeRequest::new();
        request.set_nodeType(probe::ProbeRequest_NodeType::Coordinator);
        request.set_probeType(probe::ProbeRequest_ProbeType::Liveness);

        match client.probe(request) {
            Ok(resp) => {
                if resp.available {
                    self.status = NodeStatus::Running
                } else {
                    self.status = NodeStatus::Pending
                }
            }
            Err(err) => {
                log::error!("{}", err);
                self.status = NodeStatus::Unreachable
            }
        }
    }

    fn is_available(&self) -> bool {
        self.status == NodeStatus::Running
    }
}

#[derive(Clone, Debug)]
pub struct Cluster {
    workers: Vec<Node>,
}

impl Cluster {
    pub fn partition_key<T: types::KeyedValue<K, V>, K: Hash, V>(
        &self,
        keyed: &T,
    ) -> PersistableHostAddr {
        let ref mut hasher = DefaultHasher::new();
        keyed.key().hash(hasher);

        let workers: Vec<PersistableHostAddr> = self
            .workers
            .iter()
            .filter(|worker| worker.is_available())
            .map(|node| node.host_addr.clone())
            .collect();

        if workers.is_empty() {
            return Default::default();
        }

        workers[hasher.finish() as usize % workers.len()].clone()
    }

    pub fn is_available(&self) -> bool {
        self.workers
            .iter()
            .filter(|worker| worker.is_available())
            .next()
            .is_some()
    }

    pub fn new(addrs: &Vec<NodeConfig>) -> Self {
        Cluster {
            workers: addrs.iter().map(|config| config.to_node()).collect(),
        }
    }

    pub fn probe_state(&mut self) {
        self.workers.iter_mut().for_each(|node| node.probe_state())
    }

    pub fn partition_dataflow(&self, dataflow: &Dataflow) -> HashMap<String, Dataflow> {
        dataflow
            .get_nodes()
            .iter()
            .map(|entry| {
                HashMap::from([(
                    self.partition_key(&SingleKV::new(*entry.0)),
                    vec![entry.1.clone()],
                )])
            })
            .reduce(|mut accum, mut map| {
                map.iter_mut().for_each(|entry| {
                    entry.1.iter_mut().for_each(|info| {
                        info.set_host_addr(to_host_addr(entry.0));
                    });
                    let option = accum.get_mut(entry.0);
                    if option.is_none() {
                        accum.insert(entry.0.clone(), entry.1.clone());
                    } else {
                        accum
                            .get_mut(entry.0)
                            .iter_mut()
                            .for_each(|operators| operators.append(entry.1))
                    }
                });

                accum
            })
            .map(|subgraph| {
                subgraph
                    .iter()
                    .map(|entry| {
                        (
                            entry.0.as_uri(),
                            utils::to_dataflow(dataflow.get_job_id(), entry.1, dataflow.get_meta()),
                        )
                    })
                    .collect()
            })
            .unwrap_or(Default::default())
    }

    pub fn terminate_dataflow(&self, job_id: &ResourceId) -> Result<DataflowStatus, ApiError> {
        for worker in &self.workers {
            if !worker.is_available() {
                continue;
            }
            let client = cli::new_dataflow_worker_client(cli::DataflowWorkerConfig {
                host: None,
                port: None,
                uri: Some(worker.host_addr.as_uri()),
            });
            let ref mut req = StopDataflowRequest::default();
            req.set_job_id(job_id.clone());
            match client.stop_dataflow(req) {
                Err(err) => return Err(ApiError::from(err)),
                _ => {}
            }
        }

        Ok(DataflowStatus::CLOSING)
    }

    pub fn create_dataflow(&self, dataflows: HashMap<String, Dataflow>) -> Result<(), ApiError> {
        if !self.is_available() {
            return Err(ApiError {
                code: status::CLUSTER_UNAVAILBALE,
                msg: "cluster is unavailabe".to_string(),
            });
        }

        for elem in dataflows {
            if !lang::any_match(&self.workers, |node| {
                node.host_addr.as_uri() == elem.0.clone() && node.is_available()
            }) {
                return Err(ApiError {
                    code: status::WORKER_UNAVAILABLE,
                    msg: "worker is unavailabe".to_string(),
                });
            }
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

#[derive(Clone, serde::Deserialize, Debug)]
pub struct NodeConfig {
    pub host: String,
    pub port: u16,
}

impl NodeConfig {
    fn to_node(&self) -> Node {
        Node {
            status: NodeStatus::Pending,
            host_addr: PersistableHostAddr {
                host: self.host.clone(),
                port: self.port,
            },
        }
    }
}

mod cluster_tests {
    #[test]
    pub fn test_cluster_available() {
        use super::{Cluster, NodeConfig};
        let mut cluster = Cluster::new(&vec![NodeConfig {
            host: "localhost".to_string(),
            port: 8080,
        }]);

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = super::NodeStatus::Running);

        assert!(cluster.is_available())
    }
}

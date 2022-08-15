use crate::net::{to_host_addr, PersistableHostAddr};
use crate::types;
use crate::types::SingleKV;
use crate::utils;
use proto::common::probe;
use proto::common::stream::Dataflow;
use proto::worker;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

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

use crate::collections::lang;

use crate::net::{to_host_addr, PersistableHostAddr};
use crate::types;
use crate::types::SingleKV;

use proto::common::probe_request::{NodeType, ProbeType};
use proto::common::{Dataflow, DataflowMeta, DataflowStatus, HostAddr};
use proto::common::{ProbeRequest, ResourceId};

use proto::worker::CreateSubDataflowRequest;
use proto::worker_gateway::SafeTaskWorkerRpcGateway;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::vec;

#[derive(Clone, Eq, PartialEq, Debug)]
enum NodeStatus {
    /// initialization status of node
    Pending,
    /// status if node is running
    Running,
    /// status if node is unreached
    Unreachable,
}

/// [`Node`] represents a remote task worker node.
/// Node will record all status of remote worker such as CPU, memory, I/O and liveness
#[derive(Clone, Debug)]
struct Node {
    /// The status of node
    status: NodeStatus,
    pub host_addr: PersistableHostAddr,
    pub gateway: SafeTaskWorkerRpcGateway,
}

impl Node {
    #[cfg(not(tarpaulin_include))]
    pub(crate) async fn probe_state(&mut self) {
        let request = ProbeRequest {
            node_type: NodeType::Coordinator as i32,
            probe_type: ProbeType::Liveness as i32,
        };

        match self.gateway.probe(request).await {
            Ok(resp) => {
                if resp.available {
                    self.status = NodeStatus::Running
                } else {
                    self.status = NodeStatus::Pending
                }
            }
            Err(err) => {
                tracing::error!("{}", err);
                self.status = NodeStatus::Unreachable
            }
        }
    }

    fn is_available(&self) -> bool {
        self.status == NodeStatus::Running
    }
}

/// [`Cluster`] is an abstraction of a remote cluster
/// Cluster will record status of remote workers like CPU, memory, I/O, liveness
#[derive(Clone, Debug)]
pub struct Cluster {
    /// all remote workers
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

    #[cfg(not(tarpaulin_include))]
    pub async fn probe_state(&mut self) {
        for node in &mut self.workers {
            node.probe_state().await
        }
    }

    /// A dataflow will be splitted into several partitions and deploy these sub-dataflow into different workers
    /// Graph-Partition is an NP-hard problem. Fortunately, a dataflow execution graph is too small to apply specific graph-partition algorithm
    pub fn partition_dataflow(&self, dataflow: &mut Dataflow) {
        dataflow.nodes.iter_mut().for_each(|entry| {
            let addr = self.partition_key(&SingleKV::new(*entry.0));
            if addr.is_valid() {
                entry.1.host_addr = Some(to_host_addr(&addr));
            }
        });
    }

    #[cfg(not(tarpaulin_include))]
    pub async fn terminate_dataflow(
        &mut self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, tonic::Status> {
        for worker in &mut self.workers {
            if !worker.is_available() {
                continue;
            }

            match worker.gateway.stop_dataflow(job_id.clone()).await {
                Err(err) => return Err(err),
                _ => {}
            }
        }

        Ok(DataflowStatus::Closing)
    }

    #[cfg(not(tarpaulin_include))]
    pub async fn create_dataflow(&mut self, dataflow: &Dataflow) -> Result<(), tonic::Status> {
        if !self.is_available() {
            return Err(tonic::Status::unavailable("worker is unavailable"));
        }

        let dataflows = self.split_into_subdataflow(dataflow);

        for elem in dataflows {
            let uri = elem.0.as_uri();
            if !lang::any_match(&self.workers, |node| {
                node.host_addr.as_uri() == uri && node.is_available()
            }) {
                return Err(tonic::Status::unavailable("worker is unavailable"));
            }

            for node in self
                .workers
                .iter_mut()
                .filter(|worker| worker.host_addr.as_uri() == uri)
            {
                let dataflow = Some(elem.1.clone());
                let req = CreateSubDataflowRequest {
                    job_id: elem.1.job_id.clone(),
                    dataflow,
                };

                let result = node
                    .gateway
                    .create_sub_dataflow(req)
                    .await
                    .map_err(|err| err);
                match result {
                    Ok(status) => {
                        tracing::debug!("subdataflow status: {}", status.status().as_str_name())
                    }
                    Err(err) => {
                        tracing::error!("fail to create sub dataflow {}", err);
                        return Err(err);
                    }
                }
            }
        }

        Ok(())
    }

    fn split_into_subdataflow(
        &self,
        dataflow: &Dataflow,
    ) -> HashMap<PersistableHostAddr, Dataflow> {
        let mut group = HashMap::<PersistableHostAddr, Vec<&DataflowMeta>>::new();

        dataflow.meta.iter().for_each(|node| {
            let operator = dataflow.nodes.get(&node.center).unwrap();
            let addr = operator
                .host_addr
                .as_ref()
                .map(|host_addr| PersistableHostAddr {
                    host: host_addr.host.clone(),
                    port: host_addr.port as u16,
                })
                .unwrap_or_default();

            if group.contains_key(&addr) {
                group
                    .get_mut(&addr)
                    .iter_mut()
                    .for_each(|nodes| nodes.push(node));
            } else {
                group.insert(addr, vec![node]);
            }
        });

        group
            .iter()
            .map(|entry| {
                let mut new_dataflow = Dataflow::default();
                let mut nodes = HashMap::new();
                entry.1.iter().for_each(|meta| {
                    dataflow
                        .nodes
                        .iter()
                        .filter(|entry| meta.center == *entry.0 || meta.neighbors.contains(entry.0))
                        .for_each(|entry| {
                            nodes.insert(*entry.0, entry.1.clone());
                        });
                });

                new_dataflow.meta = entry.1.iter().map(|meta| (*meta).clone()).collect();
                new_dataflow.nodes = nodes;
                new_dataflow.job_id = dataflow.job_id.clone();

                (entry.0.clone(), new_dataflow)
            })
            .collect()
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
            gateway: SafeTaskWorkerRpcGateway::new(&HostAddr {
                host: self.host.clone(),
                port: self.port as u32,
            }),
        }
    }
}

#[cfg(test)]
mod cluster_tests {

    #[tokio::test]
    pub async fn test_cluster_available() {
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

    #[tokio::test]
    pub async fn test_cluster_partition_dataflow() {
        use super::{Cluster, NodeConfig};
        use proto::common::Dataflow;
        use std::collections::HashMap;

        use proto::common::{DataflowMeta, OperatorInfo};

        use crate::net::cluster::NodeStatus;
        let mut cluster = Cluster::new(&vec![
            NodeConfig {
                host: "198.0.0.1".to_string(),
                port: 8080,
            },
            NodeConfig {
                host: "198.0.0.2".to_string(),
                port: 8080,
            },
            NodeConfig {
                host: "198.0.0.3".to_string(),
                port: 8080,
            },
        ]);
        let mut dataflow = Dataflow::default();

        let meta_1 = DataflowMeta {
            center: 0,
            neighbors: vec![1, 2, 3],
        };

        let mut nodes = HashMap::new();
        let mut op0 = OperatorInfo::default();
        op0.operator_id = 0;
        let mut op1 = OperatorInfo::default();
        op1.operator_id = 1;

        let mut op2 = OperatorInfo::default();
        op2.operator_id = 2;

        let mut op3 = OperatorInfo::default();
        op3.operator_id = 3;

        nodes.insert(0, op0);
        nodes.insert(1, op1);
        nodes.insert(2, op2);
        nodes.insert(3, op3);

        dataflow.meta = vec![meta_1];
        dataflow.nodes = nodes;

        cluster.partition_dataflow(&mut dataflow);

        dataflow.nodes.iter().for_each(|entry| {
            assert!(entry.1.host_addr.is_none());
        });

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = NodeStatus::Running);

        cluster.partition_dataflow(&mut dataflow);

        dataflow.nodes.iter().for_each(|entry| {
            assert!(entry.1.host_addr.is_some());
            let host_addr = entry.1.host_addr.as_ref().unwrap();
            assert!(!host_addr.host.is_empty());
            assert_eq!(host_addr.port, 8080);
        });
    }

    #[tokio::test]
    pub async fn test_split_into_subdataflow() {
        use super::{Cluster, NodeConfig};
        use proto::common::Dataflow;
        use std::collections::HashMap;

        use proto::common::{DataflowMeta, OperatorInfo};

        use crate::net::cluster::NodeStatus;
        let mut cluster = Cluster::new(&vec![
            NodeConfig {
                host: "198.0.0.1".to_string(),
                port: 8080,
            },
            NodeConfig {
                host: "198.0.0.2".to_string(),
                port: 8080,
            },
            NodeConfig {
                host: "198.0.0.3".to_string(),
                port: 8080,
            },
        ]);
        let mut dataflow = Dataflow::default();

        let meta_1 = DataflowMeta {
            center: 0,
            neighbors: vec![1, 2, 3],
        };

        let meta_2 = DataflowMeta {
            center: 1,
            neighbors: vec![],
        };

        let meta_3 = DataflowMeta {
            center: 2,
            neighbors: vec![],
        };

        let meta_4 = DataflowMeta {
            center: 3,
            neighbors: vec![],
        };

        let mut nodes = HashMap::new();
        let mut op0 = OperatorInfo::default();
        op0.operator_id = 0;
        let mut op1 = OperatorInfo::default();
        op1.operator_id = 1;

        let mut op2 = OperatorInfo::default();
        op2.operator_id = 2;

        let mut op3 = OperatorInfo::default();
        op3.operator_id = 3;

        nodes.insert(0, op0);
        nodes.insert(1, op1);
        nodes.insert(2, op2);
        nodes.insert(3, op3);

        dataflow.meta = vec![meta_1, meta_2, meta_3, meta_4];
        dataflow.nodes = nodes;

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = NodeStatus::Running);

        cluster.partition_dataflow(&mut dataflow);

        let result = cluster.split_into_subdataflow(&dataflow);
        assert!(!result.is_empty());

        assert_eq!(result.len(), 3);
    }
}

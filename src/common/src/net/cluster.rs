use crate::collections::lang;
use crate::types;
use crate::types::SingleKV;
use crate::utils::times::from_prost_timestamp_to_utc_chrono;

use proto::common::DataflowMeta;
use proto::common::{Dataflow, HostAddr};

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::vec;

use super::gateway::worker::SafeTaskManagerRpcGateway;

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub enum NodeStatus {
    /// initializated status of node
    Pending,
    /// status if node is running
    Running,
    /// status if node is unreached
    Unreachable,
}

/// [`Node`] represents a remote task worker node.
/// Node will record all status of remote worker such as CPU, memory, I/O and liveness
#[derive(Clone, Debug)]
pub struct Node {
    /// The status of node
    status: NodeStatus,
    /// The address of node
    pub host_addr: HostAddr,
    // the latest update time of status updating
    lastest_status_update_timestamp: chrono::DateTime<chrono::Utc>,
    // gateway of task manager
    gateway: SafeTaskManagerRpcGateway,
    /// node's id. It's always aligned with the list of [NodeBuilder]
    node_id: u32,
}

impl Node {
    pub fn new(host_addr: HostAddr, gateway: SafeTaskManagerRpcGateway) -> Self {
        Self {
            status: NodeStatus::Pending,
            host_addr,
            lastest_status_update_timestamp: chrono::Utc::now(),
            gateway,
            node_id: 0,
        }
    }

    pub fn update_status(&mut self, status: NodeStatus, timestamp: &prost_types::Timestamp) {
        self.status = status;
        self.lastest_status_update_timestamp = from_prost_timestamp_to_utc_chrono(timestamp)
    }

    #[inline]
    pub fn get_status(&self) -> &NodeStatus {
        &self.status
    }

    #[inline]
    pub fn is_available(&self) -> bool {
        self.status == NodeStatus::Running
    }

    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn get_gateway(&self) -> &SafeTaskManagerRpcGateway {
        &self.gateway
    }

    #[inline]
    pub fn get_id(&self) -> u32 {
        self.node_id
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
    pub fn get_node(&self, addr: &HostAddr) -> Option<&Node> {
        self.workers
            .iter()
            .filter(|worker| &worker.host_addr == addr)
            .next()
    }

    pub fn partition_key<T: types::KeyedValue<K, V>, K: Hash, V>(&self, keyed: &T) -> HostAddr {
        let ref mut hasher = DefaultHasher::new();
        keyed.key().hash(hasher);

        let workers: Vec<HostAddr> = self
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

    /// A dataflow will be splitted into several partitions and deploy these sub-dataflow into different workers
    /// Graph-Partition is an NP-hard problem. Fortunately, a dataflow execution graph is too small to apply specific graph-partition algorithm
    pub fn partition_dataflow(&self, dataflow: &mut Dataflow) {
        dataflow.nodes.iter_mut().for_each(|entry| {
            let addr = self.partition_key(&SingleKV::new(*entry.0));
            if addr.is_valid() {
                entry.1.host_addr = Some(addr);
            }
        });
    }

    pub fn split_into_subdataflow(&self, dataflow: &Dataflow) -> HashMap<HostAddr, Dataflow> {
        let mut group = HashMap::<HostAddr, Vec<&DataflowMeta>>::new();

        dataflow.meta.iter().for_each(|node| {
            let operator = dataflow.nodes.get(&node.center).unwrap();
            let addr = operator.host_addr.clone().unwrap_or_default();

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

#[derive(Clone, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct NodeBuilder {
    pub host: String,
    pub port: u16,
}

impl NodeBuilder {
    pub fn build(&self, gateway: SafeTaskManagerRpcGateway) -> Node {
        Node::new(
            HostAddr {
                host: self.host.clone(),
                port: self.port as u32,
            },
            gateway,
        )
    }
}

/// Builder for [Cluster]
/// It also can be used as structure of the configuration of [Cluster] in a config file.
/// Config file with types `json` and `yaml` are both supported
#[derive(Clone, serde::Deserialize, Debug)]
pub struct ClusterBuilder {
    /// task manager nodes configurations
    pub nodes: Vec<NodeBuilder>,
    /// rpc request timeout
    pub rpc_timeout: u64,
    /// rpc connection timeout
    pub connect_timeout: u64,
}

impl ClusterBuilder {
    pub fn build(&self) -> Cluster {
        Cluster {
            workers: lang::index_map(&self.nodes, |index, builder| {
                let mut node = builder.build(SafeTaskManagerRpcGateway::with_timeout(
                    &HostAddr {
                        host: builder.host.clone(),
                        port: builder.port as u32,
                    },
                    self.connect_timeout,
                    self.rpc_timeout,
                ));

                node.node_id = index as u32;
                node
            }),
        }
    }
}

#[cfg(test)]
mod cluster_tests {
    use proto::common::HostAddr;

    use crate::{
        net::{
            cluster::{ClusterBuilder, NodeBuilder},
            gateway::worker::SafeTaskManagerRpcGateway,
        },
        utils::times::prost_now,
    };

    #[tokio::test]
    pub async fn test_cluster_available() {
        use super::NodeBuilder;
        let builder = ClusterBuilder {
            nodes: vec![NodeBuilder {
                host: "localhost".to_string(),
                port: 8080,
            }],
            rpc_timeout: 3,
            connect_timeout: 3,
        };
        let mut cluster = builder.build();

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = super::NodeStatus::Running);

        assert!(cluster.is_available())
    }

    #[tokio::test]
    pub async fn test_cluster_partition_dataflow() {
        use super::NodeBuilder;
        use proto::common::Dataflow;
        use std::collections::HashMap;

        use proto::common::{DataflowMeta, OperatorInfo};

        use crate::net::cluster::NodeStatus;
        let builder = ClusterBuilder {
            nodes: vec![
                NodeBuilder {
                    host: "198.0.0.1".to_string(),
                    port: 8080,
                },
                NodeBuilder {
                    host: "198.0.0.2".to_string(),
                    port: 8080,
                },
                NodeBuilder {
                    host: "198.0.0.3".to_string(),
                    port: 8080,
                },
            ],
            rpc_timeout: 3,
            connect_timeout: 3,
        };
        let mut cluster = builder.build();
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
        use super::NodeBuilder;
        use proto::common::Dataflow;
        use std::collections::HashMap;

        use proto::common::{DataflowMeta, OperatorInfo};

        use crate::net::cluster::NodeStatus;
        let builder = ClusterBuilder {
            nodes: vec![
                NodeBuilder {
                    host: "198.0.0.1".to_string(),
                    port: 8080,
                },
                NodeBuilder {
                    host: "198.0.0.2".to_string(),
                    port: 8080,
                },
                NodeBuilder {
                    host: "198.0.0.3".to_string(),
                    port: 8080,
                },
            ],
            rpc_timeout: 3,
            connect_timeout: 3,
        };
        let mut cluster = builder.build();
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

    #[test]
    fn test_cluster_builder_derserialize() {
        let origin = "{
                \"nodes\":[
                    {
                        \"host\":\"${WORKER_1}\",
                        \"port\":8792
                    }
                ],
                \"rpc_timeout\":3,
                \"connect_timeout\":3
            }";
        std::env::set_var("WORKER_1", "localhost");
        std::env::set_var("HOME", "jason.song");
        let target = crate::utils::from_str(origin);
        let result = serde_json::from_str::<ClusterBuilder>(target.as_str());
        if result.is_err() {
            print!("{:?}", result.as_ref().unwrap_err())
        }

        let builder = result.unwrap();
        assert_eq!(builder.rpc_timeout, 3);
        assert_eq!(builder.connect_timeout, 3);
        assert_eq!(
            &builder.nodes,
            &vec![NodeBuilder {
                host: "localhost".to_string(),
                port: 8792
            }]
        )
    }

    #[tokio::test]
    async fn test_node_update_status() {
        let builder = super::NodeBuilder {
            host: "localhost".to_string(),
            port: 9999,
        };

        let mut node = builder.build(SafeTaskManagerRpcGateway::new(&HostAddr {
            host: "localhost".to_string(),
            port: 9999,
        }));

        assert_eq!(node.get_status(), &super::NodeStatus::Pending);
        let now = prost_now();

        node.update_status(super::NodeStatus::Running, &now);
        assert_eq!(node.get_status(), &super::NodeStatus::Running);
        assert!(node.is_available());
    }

    #[tokio::test]
    async fn test_cluster_build() {
        let builder = super::ClusterBuilder {
            nodes: vec![
                super::NodeBuilder {
                    host: "localhost_1".to_string(),
                    port: 9999,
                },
                super::NodeBuilder {
                    host: "localhost_2".to_string(),
                    port: 9999,
                },
            ],
            rpc_timeout: 3,
            connect_timeout: 3,
        };

        let cluster = builder.build();
        let node = cluster.get_node(&HostAddr {
            host: "localhost_1".to_string(),
            port: 9999,
        });

        assert!(node.is_some());
        let node = node.unwrap();

        assert_eq!(node.get_id(), 0);
        assert_eq!(node.get_status(), &super::NodeStatus::Pending);

        let node = cluster.get_node(&HostAddr {
            host: "localhost_2".to_string(),
            port: 9999,
        });

        assert!(node.is_some());
        let node = node.unwrap();

        assert_eq!(node.get_id(), 1);
        assert_eq!(node.get_status(), &super::NodeStatus::Pending);
    }
}

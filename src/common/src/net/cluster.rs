use crate::collections::lang;
use crate::err::ApiError;
use crate::net::{to_host_addr, PersistableHostAddr};
use crate::types;
use crate::types::SingleKV;

use proto::common::common::ResourceId;
use proto::common::probe;
use proto::common::stream::{Dataflow, DataflowMeta, DataflowStatus};
use proto::worker::worker::{CreateDataflowRequest, StopDataflowRequest};
use proto::worker::{self, cli};
use protobuf::RepeatedField;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::vec;

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

    pub fn partition_dataflow(&self, dataflow: &mut Dataflow) {
        dataflow.nodes.iter_mut().for_each(|entry| {
            let addr = self.partition_key(&SingleKV::new(*entry.0));
            if addr.is_valid() {
                entry.1.set_host_addr(to_host_addr(&addr));
            }
        });
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

    pub fn create_dataflow(&self, dataflow: Dataflow) -> Result<(), ApiError> {
        if !self.is_available() {
            return Err(ApiError {
                code: status::CLUSTER_UNAVAILBALE,
                msg: "cluster is unavailabe".to_string(),
            });
        }

        let dataflows = self.split_into_subdataflow(dataflow);

        for elem in dataflows {
            let uri = elem.0.as_uri();
            if !lang::any_match(&self.workers, |node| {
                node.host_addr.as_uri() == uri && node.is_available()
            }) {
                return Err(ApiError {
                    code: status::WORKER_UNAVAILABLE,
                    msg: "worker is unavailabe".to_string(),
                });
            }
            let client = cli::new_dataflow_worker_client(cli::DataflowWorkerConfig {
                host: None,
                port: None,
                uri: Some(uri),
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

    fn split_into_subdataflow(&self, dataflow: Dataflow) -> HashMap<PersistableHostAddr, Dataflow> {
        let mut group = HashMap::<PersistableHostAddr, Vec<&DataflowMeta>>::new();

        dataflow.get_meta().iter().for_each(|node| {
            let operator = dataflow.get_nodes().get(&node.center).unwrap();
            let addr = PersistableHostAddr {
                host: operator.get_host_addr().get_host().to_string(),
                port: operator.get_host_addr().get_port() as u16,
            };
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
                        .get_nodes()
                        .iter()
                        .filter(|entry| meta.center == *entry.0 || meta.neighbors.contains(entry.0))
                        .for_each(|entry| {
                            nodes.insert(*entry.0, entry.1.clone());
                        });
                });

                new_dataflow.set_meta(RepeatedField::from_iter(
                    entry.1.iter().map(|meta| (*meta).clone()),
                ));
                new_dataflow.set_nodes(nodes);

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

    #[test]
    pub fn test_cluster_partition_dataflow() {
        use super::{Cluster, NodeConfig};
        use proto::common::stream::Dataflow;
        use protobuf::RepeatedField;
        use std::collections::HashMap;

        use proto::common::stream::{DataflowMeta, OperatorInfo};

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

        let mut meta_1 = DataflowMeta::default();
        meta_1.set_center(0);
        meta_1.set_neighbors(vec![1, 2, 3]);

        let mut nodes = HashMap::new();
        let mut op0 = OperatorInfo::default();
        op0.set_operator_id(0);
        let mut op1 = OperatorInfo::default();
        op1.set_operator_id(1);

        let mut op2 = OperatorInfo::default();
        op2.set_operator_id(2);

        let mut op3 = OperatorInfo::default();
        op3.set_operator_id(3);

        nodes.insert(0, op0);
        nodes.insert(1, op1);
        nodes.insert(2, op2);
        nodes.insert(3, op3);

        dataflow.set_meta(RepeatedField::from_slice(&[meta_1]));
        dataflow.set_nodes(nodes);

        cluster.partition_dataflow(&mut dataflow);

        dataflow.get_nodes().iter().for_each(|entry| {
            assert!(entry.1.get_host_addr().get_host().is_empty());
            assert_eq!(entry.1.get_host_addr().get_port(), 0);
        });

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = NodeStatus::Running);

        cluster.partition_dataflow(&mut dataflow);

        dataflow.get_nodes().iter().for_each(|entry| {
            assert!(!entry.1.get_host_addr().get_host().is_empty());
            assert_eq!(entry.1.get_host_addr().get_port(), 8080);
        });
    }

    #[test]
    pub fn test_split_into_subdataflow() {
        use super::{Cluster, NodeConfig};
        use proto::common::stream::Dataflow;
        use protobuf::RepeatedField;
        use std::collections::HashMap;

        use proto::common::stream::{DataflowMeta, OperatorInfo};

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

        let mut meta_1 = DataflowMeta::default();
        meta_1.set_center(0);
        meta_1.set_neighbors(vec![1, 2, 3]);

        let mut meta_2 = DataflowMeta::default();
        meta_2.set_center(1);

        let mut meta_3 = DataflowMeta::default();
        meta_3.set_center(2);

        let mut meta_4 = DataflowMeta::default();
        meta_4.set_center(3);

        let mut nodes = HashMap::new();
        let mut op0 = OperatorInfo::default();
        op0.set_operator_id(0);
        let mut op1 = OperatorInfo::default();
        op1.set_operator_id(1);

        let mut op2 = OperatorInfo::default();
        op2.set_operator_id(2);

        let mut op3 = OperatorInfo::default();
        op3.set_operator_id(3);

        nodes.insert(0, op0);
        nodes.insert(1, op1);
        nodes.insert(2, op2);
        nodes.insert(3, op3);

        dataflow.set_meta(RepeatedField::from_slice(&[meta_1, meta_2, meta_3, meta_4]));
        dataflow.set_nodes(nodes);

        cluster
            .workers
            .iter_mut()
            .for_each(|node| node.status = NodeStatus::Running);

        cluster.partition_dataflow(&mut dataflow);

        let result = cluster.split_into_subdataflow(dataflow);
        assert!(!result.is_empty());

        assert_eq!(result.len(), 3);
    }
}

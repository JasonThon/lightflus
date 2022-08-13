use common;
use common::net::HostAddr;
use common::{err, types};
use proto::common::probe;
use proto::worker;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq, PartialEq)]
enum NodeStatus {
    Pending,
    Running,
    Unreachable,
}

#[derive(Clone, Eq, PartialEq)]
struct Node {
    status: NodeStatus,
    pub host_addr: HostAddr,
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

pub struct Cluster {
    workers: Vec<Node>,
}

impl Cluster {
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

    pub fn new(addrs: &Vec<NodeConfig>) -> Self {
        Cluster {
            workers: addrs.iter().map(|config| config.to_node()).collect(),
        }
    }

    pub fn probe_state(&mut self) {
        self.workers.iter_mut().for_each(|node| node.probe_state())
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
            host_addr: HostAddr {
                host: self.host.clone(),
                port: self.port,
            },
        }
    }
}

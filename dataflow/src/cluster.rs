use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use core;
use dataflow_api::probe;
use crate::{event, types};
use crate::cluster::NodeStatus::{Pending, Running, Unreachable};

#[derive(Clone, Eq, PartialEq)]
enum NodeStatus {
    Pending,
    Running,
    Unreachable,
}

#[derive(Clone, Eq, PartialEq)]
struct Node {
    pub addr: String,
    status: NodeStatus,
}

impl Node {
    pub(crate) fn probe_state(&mut self) {
        let client = dataflow_api::worker::new_dataflow_worker_client(dataflow_api::worker::DataflowWorkerConfig {
            host: None,
            port: None,
            uri: Some(self.addr.clone()),
        });

        let ref mut request = probe::ProbeRequest::new();
        request.set_nodeType(probe::ProbeRequest_NodeType::Coordinator);
        request.set_probeType(probe::ProbeRequest_ProbeType::Liveness);

        match client.probe(request) {
            Ok(resp) => {
                if resp.available {
                    self.status = Running
                } else {
                    self.status = Pending
                }
            }
            Err(_) => self.status = Unreachable
        }
    }

    fn send(&self, event: event::GraphEvent) -> std::io::Result<()> {
        let client = dataflow_api::worker::new_dataflow_worker_client(
            dataflow_api::worker::DataflowWorkerConfig {
                host: None,
                port: None,
                uri: Some(self.addr.clone()),
            }
        );
        match serde_json::to_string(&event) {
            Ok(data) => {
                let ref mut req = dataflow_api::dataflow_worker::ActionSubmitRequest::new();
                req.set_value(data.as_bytes().to_vec());

                match client.submit_action(req) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(core::http::to_io_error(err))
                }
            }
            Err(err) => Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("serialize event failed: {:?}", err),
                )
            )
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
    pub fn partition_key<T: core::KeyedValue<K, V>, K: Hash, V>(&self, keyed: &T) -> String {
        let ref mut hasher = DefaultHasher::new();
        keyed.key().hash(hasher);
        let workers = core::lists::filter_map(
            &self.workers,
            |worker| worker.is_available(),
            |node| node.addr.clone(),
        );

        workers[hasher.finish() as usize % workers.len()].clone()
    }

    pub fn new(addrs: &Vec<NodeConfig>) -> Self {
        Cluster {
            workers: core::lists::map(
                addrs,
                |addr| addr.to_node(),
            ),
        }
    }

    pub(crate) fn stop_job(&self, job_id: &types::JobID) -> std::io::Result<()> {
        for worker in &self.workers {
            if worker.is_available() {
                match worker.send(event::GraphEvent::StopGraph { job_id: job_id.clone() }) {
                    Err(err) => return Err(err),
                    Ok(_) => continue
                }
            }
        }

        Ok(())
    }

    pub fn probe_state(&mut self) {
        core::lists::for_each_mut(&mut self.workers, |node| {
            node.probe_state()
        })
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct NodeConfig {
    pub host: String,
    pub port: u16,
}

impl NodeConfig {
    fn to_node(&self) -> Node {
        Node {
            addr: format!("{}:{}", &self.host, &self.port),
            status: NodeStatus::Pending,
        }
    }
}
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use common;
use common::{err, event, types};
use proto::common::probe;
use proto::worker;

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
        let client = worker::cli::new_dataflow_worker_client(worker::cli::DataflowWorkerConfig {
            host: None,
            port: None,
            uri: Some(self.addr.clone()),
        });

        let ref mut request = probe::ProbeRequest::new();
        request.set_nodeType(probe::probe_request::NodeType::Coordinator);
        request.set_probeType(probe::probe_request::ProbeType::Liveness);

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
    pub fn partition_key<T: types::KeyedValue<K, V>, K: Hash, V>(&self, keyed: &T) -> Result<String, err::CommonException> {
        let ref mut hasher = DefaultHasher::new();
        keyed.key().hash(hasher);
        let workers = common::lists::filter_map(
            &self.workers,
            |worker| worker.is_available(),
            |node| node.addr.clone(),
        );

        if workers.is_empty() {
            return Err(err::CommonException::new(err::ErrorKind::NoAvailableWorker, "no available worker"));
        }

        Ok(workers[hasher.finish() as usize % workers.len()].clone())
    }

    pub fn is_available(&self) -> bool {
        common::lists::any_match(&self.workers, |worker| worker.is_available())
    }

    pub fn new(addrs: &Vec<NodeConfig>) -> Self {
        Cluster {
            workers: common::lists::map(
                addrs,
                |addr| addr.to_node(),
            ),
        }
    }

    pub(crate) fn stop_job(&self, job_id: &types::JobID) -> Result<(), grpcio::Error> {
        for worker in &self.workers {
            if worker.is_available() {
                let cli = worker::cli::new_dataflow_worker_client(worker::cli::DataflowWorkerConfig {
                    host: None,
                    port: None,
                    uri: Some(worker.addr.clone()),
                });
                let mut req = worker::worker::StopStreamGraphRequest::default();
                req.job_id = ::protobuf::MessageField::some(job_id.into());

                match cli.stop_stream_graph(&req) {
                    Err(err) => return Err(err),
                    Ok(_) => continue
                }
            }
        }

        Ok(())
    }

    pub fn probe_state(&mut self) {
        common::lists::for_each_mut(&mut self.workers, |node| {
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
use std::fs;

use common::net::cluster;
use common::net::AckResponderBuilder;
use common::net::HeartbeatBuilder;
use common::utils;
use proto::common::Ack;
use proto::common::Dataflow;
use proto::common::DataflowStates;
use proto::common::DataflowStatus;

use proto::common::Heartbeat;
use proto::common::NodeType;
use proto::common::ResourceId;

use super::managers::Dispatcher;
use super::storage::DataflowStorageBuilder;

/// Builder for [Coordinator]
/// It's also the configuration of Coordinator. You can see in the file `etc/coord.json`
#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorBuilder {
    /// Coordinator port
    pub port: usize,
    /// TaskManager Cluster builder
    pub cluster: cluster::ClusterBuilder,
    /// dataflow storage builder
    pub storage: DataflowStorageBuilder,
    /// heartbeat builder
    pub heartbeat: HeartbeatBuilder,
    // ack responder builder
    pub ack: AckResponderBuilder,
}

impl CoordinatorBuilder {
    pub fn build(&self) -> Coordinator {
        Coordinator {
            dispatcher: Dispatcher::new(
                &self.cluster,
                &self.storage,
                &self.heartbeat,
                &self.ack,
                self.port,
            ),
        }
    }
}

pub fn load_builder() -> CoordinatorBuilder {
    serde_json::from_str::<CoordinatorBuilder>(
        utils::from_reader(
            fs::File::open(
                utils::Args::default()
                    .arg("c")
                    .map(|arg| arg.value.clone())
                    .unwrap_or("src/coordinator/etc/coord.json".to_string()),
            )
            .expect("fail to read config file: "),
        )
        .expect("fail to read config file: ")
        .as_str(),
    )
    .expect("fail to parser config file: ")
}

/// The coordinator of a Lightflus cluster
/// Coordinator will manage:
/// - [Dispatcher]
/// - Checkpoint Coordinator
/// - Backpressure Metrics
/// - Scale Up and Scale Down
pub struct Coordinator {
    dispatcher: Dispatcher,
}

impl Coordinator {
    pub(crate) async fn create_dataflow(&self, dataflow: Dataflow) -> Result<(), tonic::Status> {
        match dataflow
            .validate()
            .map_err(|err| tonic::Status::invalid_argument(format!("{:?}", err)))
        {
            Ok(_) => {
                let terminate_result = self
                    .terminate_dataflow(dataflow.job_id.as_ref().unwrap())
                    .await;
                if terminate_result.is_err() {
                    return terminate_result.map(|_| ());
                }
                self.dispatcher
                    .create_dataflow(dataflow)
                    .await
                    .map_err(|err| err.to_tonic_status())
            }
            Err(err) => Err(err),
        }
    }

    pub(crate) async fn terminate_dataflow(
        &self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, tonic::Status> {
        self.dispatcher
            .terminate_dataflow(job_id)
            .await
            .map_err(|err| err.to_tonic_status())
    }

    pub(crate) async fn get_dataflow(
        &self,
        job_id: &ResourceId,
    ) -> Result<DataflowStates, tonic::Status> {
        self.dispatcher
            .get_dataflow(job_id)
            .await
            .map_err(|err| err.to_tonic_status())
    }

    pub(crate) async fn receive_heartbeart(&self, heartbeat: &Heartbeat) {
        self.dispatcher
            .update_task_manager_heartbeat_status(heartbeat)
            .await
    }

    pub(crate) async fn receive_ack(&self, ack: Ack) {
        match ack.node_type() {
            NodeType::TaskWorker => self.dispatcher.ack_from_task_manager(ack).await,
            _ => {}
        }
    }
}

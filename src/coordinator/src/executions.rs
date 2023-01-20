use std::collections::BTreeMap;

use common::{
    net::{
        cluster::{Node, NodeStatus},
        to_host_addr, AckResponder, AckResponderBuilder, PersistableHostAddr,
    },
    types::ExecutorId,
    utils::{self, times::from_utc_chrono_to_prost_timestamp},
    ExecutionID,
};
use proto::{
    common::{
        ack::{AckType, RequestId},
        Ack, Dataflow, DataflowStatus, Heartbeat, NodeType, OperatorInfo,
    },
    worker::CreateSubDataflowRequest,
    worker_gateway::SafeTaskManagerRpcGateway,
};
use tokio::task::JoinHandle;

/// A [`VertexExecution`] represents an execution context for a [`LocalExecutor`].
/// - watch each LocalExecutor's state details
/// - restart LocalExecutor while it stops unexpectedly
/// - watch each LocalExecutor's checkpoint snapshot status
/// - collect each LocalExecutor's metrics
pub struct VertexExecution {
    executor_id: ExecutorId,
    operator: OperatorInfo,
}

impl VertexExecution {
    pub(crate) fn new(executor_id: &ExecutorId, operator: &OperatorInfo) -> Self {
        todo!()
    }
}

/// A [`SubdataflowDeploymentPlan`] represents a description for a subdataflow [`Dataflow`] deployment. It may contains following properties:
/// - the structure of subdataflow
/// - the execution id of the subdataflow
/// - the resource configurations that this dataflow can be allocated
/// - ack responder
/// - initialized ack request queue
pub(crate) struct SubdataflowDeploymentPlan {
    /// the description of subdataflow
    subdataflow: Dataflow,
    /// the target address of TaskManager
    addr: PersistableHostAddr,
    /// the id of the subdataflow's execution
    execution_id: ExecutionID,
    /// the gateway of TaskManager
    gateway: Option<SafeTaskManagerRpcGateway>,
    /// ack responder
    ack: AckResponder<SafeTaskManagerRpcGateway>,
    /// the initialized enqueue-entrypoint of a ack request queue
    sender: tokio::sync::mpsc::Sender<Ack>,
}
impl SubdataflowDeploymentPlan {
    pub(crate) fn new(
        subdataflow: (&PersistableHostAddr, &Dataflow),
        execution_id: ExecutionID,
        node: Option<&Node>,
        ack_builder: &AckResponderBuilder,
    ) -> Self {
        let (ack, sender) = ack_builder.build(|addrs| {
            addrs
                .iter()
                .map(|addr| SafeTaskManagerRpcGateway::new(&to_host_addr(addr)))
                .collect()
        });
        Self {
            subdataflow: subdataflow.1.clone(),
            addr: subdataflow.0.clone(),
            execution_id,
            gateway: node.map(|n| n.gateway.clone()),
            ack,
            sender,
        }
    }

    pub(crate) async fn deploy(self) -> Result<SubdataflowExecution, TaskDeploymentException> {
        let ack = self.ack;
        match &self.gateway {
            Some(gateway) => {
                let req = CreateSubDataflowRequest {
                    job_id: Some(self.subdataflow.get_job_id()),
                    dataflow: Some(self.subdataflow.clone()),
                };
                let _ = self.subdataflow.nodes.iter().map(|(executor_id, info)| {});

                match gateway.create_sub_dataflow(req).await {
                    Ok(resp) => Ok(SubdataflowExecution {
                        worker: Node::new(self.addr.clone(), gateway.clone()),
                        vertexes: self
                            .subdataflow
                            .nodes
                            .iter()
                            .map(|(executor_id, info)| {
                                (*executor_id, VertexExecution::new(executor_id, info))
                            })
                            .collect(),
                        execution_id: self.execution_id.clone(),
                        status: resp.status(),
                        ack_handler: tokio::spawn(ack),
                        ack_request_queue: self.sender,
                    }),
                    Err(err) => Err(TaskDeploymentException::RpcError(err)),
                }
            }
            None => Err(TaskDeploymentException::InvalidWorkerEndpoint),
        }
    }
}

pub(crate) enum TaskDeploymentException {
    InvalidWorkerEndpoint,
    RpcError(tonic::Status),
}

/// A [`SubdataflowExecution`] represents a execution context of a subdataflow. It's responsible for:
/// - watch the status of subdataflow
/// - send heartbeat ack to TaskWorker
/// - store basic information of a subdataflow
/// - manage the checkpoint snapshot of a subdataflow
/// - manage the checkpoint trigger of a subdataflow
pub(crate) struct SubdataflowExecution {
    /// the remote TaskManager node
    worker: Node,
    /// all vertexes execution contexts
    vertexes: BTreeMap<ExecutorId, VertexExecution>,
    /// the id of the subdataflow execution
    execution_id: ExecutionID,
    /// the status of subdataflow
    status: DataflowStatus,
    /// the asynchronous task of the ack sender
    ack_handler: JoinHandle<()>,
    /// the enqueue-entrypoint of a ack request queue
    ack_request_queue: tokio::sync::mpsc::Sender<Ack>,
}
impl SubdataflowExecution {
    pub(crate) fn try_terminate(&mut self) {
        todo!()
    }

    pub(crate) fn get_execution_id(&self) -> &ExecutionID {
        &self.execution_id
    }

    pub(crate) async fn update_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        match heartbeat.timestamp.as_ref() {
            Some(timestamp) => match heartbeat.node_type() {
                NodeType::TaskWorker => {
                    self.worker.update_status(NodeStatus::Running, timestamp);
                    let ref now = utils::times::now();
                    let _ = self
                        .ack_request_queue
                        .send(Ack {
                            timestamp: Some(from_utc_chrono_to_prost_timestamp(now)),
                            ack_type: AckType::Heartbeat as i32,
                            node_type: NodeType::JobManager as i32,
                            execution_id: Some(self.execution_id.into_prost()),
                            request_id: Some(RequestId::HeartbeatId(heartbeat.heartbeat_id)),
                        })
                        .await;
                }
                _ => {}
            },
            None => {}
        }
    }
}

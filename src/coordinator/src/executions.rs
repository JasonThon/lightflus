use std::collections::BTreeMap;

use common::{
    net::{
        cluster::{Node, NodeStatus},
        to_host_addr, AckResponder, AckResponderBuilder, PersistableHostAddr,
    },
    types::ExecutorId,
};
use proto::{
    common::{
        Ack, Dataflow, DataflowStatus, ExecutionId, Heartbeat, NodeType, OperatorInfo, ResourceId,
    },
    worker::CreateSubDataflowRequest,
    worker_gateway::SafeTaskManagerRpcGateway,
};
use tokio::task::JoinHandle;

#[derive(
    Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub(crate) struct ExecutionID(pub ResourceId, pub u32);

impl From<&ExecutionId> for ExecutionID {
    fn from(id: &ExecutionId) -> Self {
        Self(id.get_job_id(), id.sub_id)
    }
}

impl ExecutionID {
    pub fn into_prost(&self) -> ExecutionId {
        ExecutionId {
            job_id: Some(self.0.clone()),
            sub_id: self.1,
        }
    }
}

/// A [`VertexExecution`] represents an execution context for a [`LocalExecutor`].
/// - watch each LocalExecutor's state details
/// - restart LocalExecutor while it stops unexpectedly
/// -
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
pub(crate) struct SubdataflowDeploymentPlan {
    subdataflow: Dataflow,
    addr: PersistableHostAddr,
    execution_id: ExecutionID,
    gateway: Option<SafeTaskManagerRpcGateway>,
    ack: AckResponder<SafeTaskManagerRpcGateway>,
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

pub(crate) struct SubdataflowExecution {
    worker: Node,
    vertexes: BTreeMap<ExecutorId, VertexExecution>,
    execution_id: ExecutionID,
    status: DataflowStatus,
    ack_handler: JoinHandle<()>,
    ack_request_queue: tokio::sync::mpsc::Sender<Ack>,
}
impl SubdataflowExecution {
    pub(crate) fn try_terminate(&mut self) {
        todo!()
    }

    pub(crate) fn get_execution_id(&self) -> &ExecutionID {
        &self.execution_id
    }

    pub(crate) fn update_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        heartbeat
            .timestamp
            .as_ref()
            .iter()
            .for_each(|timestamp| match heartbeat.node_type() {
                // TODO should ack after update status
                NodeType::TaskWorker => {
                    self.worker.update_status(NodeStatus::Running, *timestamp);
                }
                _ => {}
            })
    }
}

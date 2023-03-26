use std::{
    collections::BTreeMap,
    sync::atomic::{self, AtomicU64},
};

use common::{
    net::{
        cluster::{Node, NodeStatus},
        gateway::{
            taskmanager::SafeTaskManagerRpcGateway, ReceiveAckRpcGateway,
            ReceiveHeartbeatRpcGateway,
        },
        AckResponder, AckResponderBuilder, HeartbeatBuilder, HeartbeatSender,
    },
    types::ExecutorId,
    utils::{self, times::from_utc_chrono_to_prost_timestamp},
};
use proto::{
    common::{
        ack::{AckType, RequestId},
        Ack, Dataflow, DataflowStatus, Heartbeat, HostAddr, NodeType, OperatorInfo, ResourceId,
        SubDataflowId,
    },
    taskmanager::CreateSubDataflowRequest,
};
use tokio::{sync::mpsc, task::JoinHandle};

/// This module contains all logical execution contexts of a dataflow, an operator or an edge which are running on the remote TaskManager node.
/// These contexts contains data which can reflect the inner state of the dataflows, operators and edges such as running or not, checkpoint status.
///
/// # Observability
///
/// Execution must be observable in a cloud environment to help developers to know what happens in a running dataflow or operator.
/// If anything wrong happens, they can be informed as soon as possible and take the actions.
///
/// Mainstream cloud-observability systems like Prometheus are using three kinds of data:
/// - Metrics
/// - Tracing
/// - Logs
///
/// In 1.0 release version, Lightflus will periodically request/report these data and store them in Coordinator/TaskManger.
/// Users can configure to dump them into an outside system like ES automatically.
///
/// # Fault Tolerance
///
/// # High Availability
///
///

/// A [`VertexExecution`] represents an execution context for a [`LocalExecutor`].
/// - watch each LocalExecutor's state details
/// - restart LocalExecutor while it stops unexpectedly
/// - watch each LocalExecutor's checkpoint snapshot status
/// - collect each LocalExecutor's metrics
pub(crate) struct VertexExecution {
    executor_id: ExecutorId,
    operator: OperatorInfo,
    /// the asynchronous task of the ack sender
    ack_handler: JoinHandle<()>,
    /// the enqueue-entrypoint of a ack request queue
    ack_request_queue: mpsc::Sender<Ack>,
    // the asynchronous task of the heartbeat sender
    heartbeat_handler: JoinHandle<()>,
    /// the latest heartbeat ack id
    latest_ack_heartbeat_id: AtomicU64,
    /// the latest heartbeat timestamp
    latest_ack_heartbeat_timestamp: AtomicU64,
}

impl VertexExecution {
    pub(crate) fn new(
        executor_id: ExecutorId,
        operator: &OperatorInfo,
        ack_builder: &AckResponderBuilder,
        heartbeat_builder: &HeartbeatBuilder,
    ) -> Self {
        let host_addr = operator.get_host_addr();
        let (ack, sender) = ack_builder.build(&host_addr, |addr, connect_timeout, rpc_timout| {
            SafeTaskManagerRpcGateway::with_timeout(addr, connect_timeout, rpc_timout)
        });

        let heartbeat =
            heartbeat_builder.build(&host_addr, |host_addr, connect_timeout, rpc_timeout| {
                SafeTaskManagerRpcGateway::with_timeout(host_addr, connect_timeout, rpc_timout)
            });
        Self {
            executor_id,
            operator: operator.clone(),
            heartbeat_handler: tokio::spawn(heartbeat),
            ack_handler: tokio::spawn(ack),
            ack_request_queue: sender,
            latest_ack_heartbeat_id: Default::default(),
            latest_ack_heartbeat_timestamp: Default::default(),
        }
    }
}

/// A [`SubdataflowDeploymentPlan`] represents a description for a subdataflow [`Dataflow`] deployment. It may contains following properties:
/// - the structure of subdataflow
/// - the execution id of the subdataflow
/// - the resource configurations that this dataflow can be allocated
/// - ack responder
/// - initialized ack request queue
pub(crate) struct SubdataflowDeploymentPlan<'a> {
    /// the description of subdataflow
    subdataflow: Dataflow,
    /// the target address of TaskManager
    addr: HostAddr,
    /// the job id of the subdataflow's execution
    job_id: ResourceId,
    /// the node of TaskManager
    node: Option<&'a Node>,
    /// ack responder
    ack: &'a AckResponderBuilder,
    // heartbeat sender
    heartbeat: &'a HeartbeatBuilder,
}

impl<'a> SubdataflowDeploymentPlan<'a> {
    pub(crate) fn new(
        subdataflow: (&HostAddr, &Dataflow),
        job_id: &ResourceId,
        node: Option<&'a Node>,
        ack_builder: &AckResponderBuilder,
        heartbeat_builder: &HeartbeatBuilder,
    ) -> Self {
        Self {
            subdataflow: subdataflow.1.clone(),
            addr: subdataflow.0.clone(),
            job_id: job_id.clone(),
            node,
            ack: ack_builder,
            heartbeat: heartbeat_builder,
        }
    }

    #[inline]
    pub(crate) async fn deploy(mut self) -> Result<SubdataflowExecution, TaskDeploymentException> {
        match &self.node {
            Some(node) => {
                self.subdataflow.execution_id = Some(SubDataflowId {
                    job_id: Some(self.job_id.clone()),
                    sub_id: node.get_id(),
                });

                let req = CreateSubDataflowRequest {
                    job_id: Some(self.subdataflow.get_job_id()),
                    dataflow: Some(self.subdataflow.clone()),
                };

                match node.get_gateway().create_sub_dataflow(req).await {
                    Ok(resp) => Ok(SubdataflowExecution::new(
                        (*node).clone(),
                        self.subdataflow,
                        SubDataflowId {
                            job_id: Some(self.job_id.clone()),
                            sub_id: node.get_id(),
                        },
                        resp.status(),
                        self.ack,
                        self.heartbeat,
                    )),
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
    execution_id: SubDataflowId,
    /// the status of subdataflow
    status: DataflowStatus,
}
impl SubdataflowExecution {
    pub(crate) fn new(
        worker: Node,
        subdataflow: Dataflow,
        execution_id: SubDataflowId,
        status: DataflowStatus,
        ack: &AckResponderBuilder,
        heartbeat: &HeartbeatBuilder,
    ) -> Self {
        let addr = worker.host_addr.clone();
        Self {
            worker,
            vertexes: subdataflow
                .nodes
                .iter()
                .map(|(executor_id, info)| {
                    (
                        *executor_id,
                        VertexExecution::new(*executor_id, info, ack, heartbeat),
                    )
                })
                .collect(),
            execution_id,
            status,
        }
    }

    pub(crate) fn try_terminate(&mut self) {
        todo!()
    }

    pub(crate) fn get_execution_id(&self) -> &SubDataflowId {
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
                            execution_id: Some(self.execution_id.clone()),
                            request_id: Some(RequestId::HeartbeatId(heartbeat.heartbeat_id)),
                        })
                        .await;
                }
                _ => {}
            },
            None => {}
        }
    }

    pub(crate) fn ack(&mut self, ack: &Ack) {
        match ack.ack_type() {
            AckType::Heartbeat => {
                if let Some(&RequestId::HeartbeatId(heartbeat_id)) = ack.request_id.as_ref() {
                    self.status = DataflowStatus::Running;
                    if self.latest_ack_heartbeat_id.load(atomic::Ordering::Relaxed) < heartbeat_id {
                        self.latest_ack_heartbeat_id
                            .swap(heartbeat_id, atomic::Ordering::AcqRel);
                        ack.timestamp.as_ref().iter().for_each(|timestamp| {
                            self.latest_ack_heartbeat_timestamp
                                .swap(timestamp.seconds as u64, atomic::Ordering::AcqRel);
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{self, AtomicU64};

    use common::{
        net::{
            cluster::{Node, NodeStatus},
            gateway::{taskmanager::SafeTaskManagerRpcGateway, MockRpcGateway},
            AckResponderBuilder,
        },
        utils::times::prost_now,
    };
    use proto::common::{
        ack::{AckType, RequestId},
        Ack, DataflowStatus, Heartbeat, HostAddr, NodeType, SubDataflowId,
    };

    #[tokio::test]
    async fn test_subdataflow_execution_update_heartbeat_status() {
        let ack_responder_builder = AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            nodes: vec![HostAddr::default()],
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let (gateway, mut ack_rx, _) = MockRpcGateway::new(ack_responder_builder.buf_size, 10);

        let (ack_responder, ack_tx) = ack_responder_builder.build(|_, _, _| gateway.clone());

        let mut execution = super::SubdataflowExecution {
            worker: Node::new(
                HostAddr::default(),
                SafeTaskManagerRpcGateway::new(&HostAddr::default()),
            ),
            vertexes: Default::default(),
            execution_id: Default::default(),
            status: DataflowStatus::Initialized,
            ack_handler: tokio::spawn(ack_responder),
            ack_request_queue: ack_tx,
            latest_ack_heartbeat_id: AtomicU64::default(),
            latest_ack_heartbeat_timestamp: AtomicU64::default(),
        };

        execution
            .update_heartbeat_status(&Heartbeat {
                heartbeat_id: 1,
                timestamp: Some(prost_now()),
                node_type: NodeType::TaskWorker as i32,
                subdataflow_id: Some(SubDataflowId {
                    job_id: Some(Default::default()),
                    sub_id: 0,
                }),
            })
            .await;

        assert_eq!(execution.worker.get_status(), &NodeStatus::Running);
        let option = ack_rx.recv().await;
        assert!(option.is_some());

        let result = option.unwrap();

        assert_eq!(result.request_id, Some(RequestId::HeartbeatId(1)));
        assert_eq!(result.ack_type(), AckType::Heartbeat);
        assert_eq!(result.node_type(), NodeType::JobManager);
        assert_eq!(
            result.execution_id,
            Some(SubDataflowId {
                job_id: None,
                sub_id: 0,
            })
        );
    }

    #[tokio::test]
    async fn test_subdataflow_execution_ack_heartbeat() {
        let ack_responder_builder = AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            nodes: vec![HostAddr::default()],
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let (gateway, _, _) = MockRpcGateway::new(ack_responder_builder.buf_size, 10);

        let (ack_responder, ack_tx) = ack_responder_builder.build(|_, _, _| gateway.clone());

        let mut execution = super::SubdataflowExecution {
            worker: Node::new(
                HostAddr::default(),
                SafeTaskManagerRpcGateway::new(&HostAddr::default()),
            ),
            vertexes: Default::default(),
            execution_id: Default::default(),
            status: DataflowStatus::Initialized,
            ack_handler: tokio::spawn(ack_responder),
            ack_request_queue: ack_tx,
            latest_ack_heartbeat_id: AtomicU64::default(),
            latest_ack_heartbeat_timestamp: AtomicU64::default(),
        };
        let now = prost_now();

        {
            execution.ack(&Ack {
                timestamp: Some(now.clone()),
                ack_type: AckType::Heartbeat as i32,
                node_type: NodeType::TaskWorker as i32,
                execution_id: Some(SubDataflowId {
                    job_id: Default::default(),
                    sub_id: 1,
                }),
                request_id: Some(RequestId::HeartbeatId(2)),
            });

            assert_eq!(
                execution
                    .latest_ack_heartbeat_id
                    .load(atomic::Ordering::Relaxed),
                2
            );
            assert_eq!(
                execution
                    .latest_ack_heartbeat_timestamp
                    .load(atomic::Ordering::Relaxed),
                now.seconds as u64
            );
        }

        let now_1 = prost_now();
        {
            execution.ack(&Ack {
                timestamp: Some(now_1.clone()),
                ack_type: AckType::Heartbeat as i32,
                node_type: NodeType::TaskWorker as i32,
                execution_id: Some(SubDataflowId {
                    job_id: Default::default(),
                    sub_id: 1,
                }),
                request_id: Some(RequestId::HeartbeatId(1)),
            });

            assert_eq!(
                execution
                    .latest_ack_heartbeat_id
                    .load(atomic::Ordering::Relaxed),
                2
            );
            assert_eq!(
                execution
                    .latest_ack_heartbeat_timestamp
                    .load(atomic::Ordering::Relaxed),
                now.seconds as u64
            );
        }
    }
}
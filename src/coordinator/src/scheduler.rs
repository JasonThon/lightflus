use std::collections::BTreeMap;

use common::{
    net::{gateway::worker::SafeTaskManagerRpcGateway, HeartbeatBuilder},
    ExecutionID,
};
use mockall::automock;
use proto::common::DataflowStatus;
use tokio::task::JoinHandle;

use crate::executions::{SubdataflowDeploymentPlan, SubdataflowExecution, TaskDeploymentException};

/// The scheduler for a [`JobManager`].
pub(crate) struct Scheduler {
    executions: BTreeMap<ExecutionID, SubdataflowExecution>,
    heartbeat_handlers: BTreeMap<ExecutionID, JoinHandle<()>>,
}

#[automock]
impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            executions: Default::default(),
            heartbeat_handlers: Default::default(),
        }
    }

    pub(crate) async fn execute<'a>(
        &'a mut self,
        plan: SubdataflowDeploymentPlan<'a>,
        heartbeat_builder: &HeartbeatBuilder,
    ) -> Result<(), TaskDeploymentException> {
        plan.deploy().await.map(|execution| {
            let execution_id = execution.get_execution_id().clone();
            self.executions.insert(execution_id.clone(), execution);

            self.heartbeat_handlers.insert(
                execution_id,
                tokio::spawn(
                    heartbeat_builder.build(|host_addr, connect_timeout, rpc_timeout| {
                        SafeTaskManagerRpcGateway::with_timeout(
                            host_addr,
                            connect_timeout,
                            rpc_timeout,
                        )
                    }),
                ),
            );
        })
    }

    pub(crate) async fn terminate_dataflow(
        &mut self,
    ) -> Result<DataflowStatus, TaskExecutionException> {
        for (_, execution) in self.executions.iter_mut() {
            execution.try_terminate()
        }

        Ok(DataflowStatus::Closing)
    }

    pub(crate) fn get_execution_mut<'a>(
        &'a mut self,
        execution_id: &ExecutionID,
    ) -> Option<&'a mut SubdataflowExecution> {
        self.executions.get_mut(execution_id)
    }
}

#[derive(Debug)]
pub(crate) enum TaskExecutionException {}

impl TaskExecutionException {
    pub(crate) fn to_tonic_status(&self) -> tonic::Status {
        todo!()
    }
}

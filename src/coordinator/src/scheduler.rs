use std::collections::BTreeMap;

use common::net::{cluster::Cluster, HeartbeatBuilder};
use proto::common::DataflowStatus;
use tokio::task::JoinHandle;

use crate::executions::{
    ExecutionID, SubdataflowDeploymentPlan, SubdataflowExecution, TaskDeploymentException,
};

#[derive(Default)]
pub struct Scheduler {
    executions: BTreeMap<ExecutionID, SubdataflowExecution>,
    heartbeat_handlers: BTreeMap<ExecutionID, JoinHandle<()>>,
}

impl Scheduler {
    pub(crate) async fn terminate_dataflow(
        &mut self,
        cluster: &mut Cluster,
    ) -> Result<DataflowStatus, TaskExecutionException> {
        todo!()
    }

    pub(crate) async fn execute_all<I: Iterator<Item = SubdataflowDeploymentPlan>>(
        &mut self,
        plan_iter: I,
        heartbeat_builder: &HeartbeatBuilder,
    ) -> Result<(), TaskDeploymentException> {
        let execution_futures = plan_iter.map(|plan| plan.deploy());

        for future in execution_futures {
            match future.await {
                Ok(execution) => {
                    let execution_id = execution.get_execution_id().clone();
                    self.executions.insert(execution_id.clone(), execution);

                    let mut heartbeat = heartbeat_builder.build();
                    heartbeat.update_execution_id(execution_id.into_prost());

                    self.heartbeat_handlers
                        .insert(execution_id, tokio::spawn(heartbeat));
                }
                Err(err) => {
                    self.executions
                        .iter_mut()
                        .for_each(|(_, execution)| execution.try_terminate());

                    self.heartbeat_handlers
                        .iter()
                        .for_each(|(_, handler)| handler.abort());
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn get_execution_mut(
        &mut self,
        execution_id: ExecutionID,
    ) -> Option<&mut SubdataflowExecution> {
        self.executions.get_mut(&execution_id)
    }
}

pub(crate) enum TaskExecutionException {}

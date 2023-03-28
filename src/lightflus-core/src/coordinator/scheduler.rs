use crossbeam_skiplist::SkipMap;
use proto::common::{Ack, DataflowStatus, Heartbeat, SubDataflowId};

use super::executions::{SubdataflowDeploymentPlan, SubdataflowExecution, TaskDeploymentException};

/// The scheduler for a [`JobManager`].
pub(crate) struct Scheduler {
    executions: SkipMap<SubDataflowId, SubdataflowExecution>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            executions: Default::default(),
        }
    }

    pub(crate) async fn execute<'a>(
        &'a mut self,
        plan: SubdataflowDeploymentPlan<'a>,
    ) -> Result<(), TaskDeploymentException> {
        plan.deploy().await.map(|execution| {
            let execution_id = execution.get_execution_id().clone();
            self.executions.insert(execution_id.clone(), execution);
        })
    }

    pub(crate) async fn terminate_dataflow(
        &self,
    ) -> Result<DataflowStatus, TaskExecutionException> {
        for entry in self.executions.iter() {
            entry.value().try_terminate();
        }

        Ok(DataflowStatus::Closing)
    }

    pub(crate) async fn receive_heartbeat(&self, heartbeat: &Heartbeat) {
        match heartbeat
            .get_execution_id()
            .and_then(|execution_id| self.executions.get(execution_id))
        {
            Some(entry) => entry.value().update_heartbeat_status(heartbeat).await,
            None => {}
        }
    }

    pub(crate) fn ack(&self, ack: &Ack) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) enum TaskExecutionException {}

impl TaskExecutionException {
    pub(crate) fn to_tonic_status(&self) -> tonic::Status {
        todo!()
    }
}

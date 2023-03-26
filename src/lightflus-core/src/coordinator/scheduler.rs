use std::collections::BTreeMap;

use proto::common::{DataflowStatus, SubDataflowId};

use super::executions::{SubdataflowDeploymentPlan, SubdataflowExecution, TaskDeploymentException};

/// The scheduler for a [`JobManager`].
pub(crate) struct Scheduler {
    executions: BTreeMap<SubDataflowId, SubdataflowExecution>,
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
        &mut self,
    ) -> Result<DataflowStatus, TaskExecutionException> {
        for (_, execution) in self.executions.iter_mut() {
            execution.try_terminate()
        }

        Ok(DataflowStatus::Closing)
    }

    pub(crate) fn get_execution_mut<'a>(
        &'a mut self,
        execution_id: &SubDataflowId,
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

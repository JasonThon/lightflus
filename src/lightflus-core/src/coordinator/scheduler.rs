use crossbeam_skiplist::SkipMap;
use proto::common::{
    Ack, Dataflow, DataflowStates, DataflowStatus, Heartbeat, SubDataflowId, SubdataflowInfo,
};

use super::executions::{
    SubdataflowDeploymentPlan, SubdataflowError, SubdataflowExecution, TaskDeploymentException,
};

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
            .get_subdataflow_id()
            .and_then(|execution_id| self.executions.get(execution_id))
        {
            Some(entry) => entry.value().update_heartbeat_status(heartbeat).await,
            None => {}
        }
    }

    pub(crate) fn ack(&self, ack: &Ack) {
        todo!()
    }

    pub async fn get_dataflow(&self, dataflow: &Dataflow) -> DataflowStates {
        let mut states = DataflowStates {
            graph: Some(dataflow.clone()),
            subdataflow_infos: vec![],
            status: DataflowStatus::Initialized as i32,
        };

        for entry in &self.executions {
            let execution = entry.value();
            match execution.get_states().await {
                Ok(subdataflow_states) => subdataflow_states
                    .subdataflow_infos
                    .into_iter()
                    .for_each(|infos| states.subdataflow_infos.push(infos)),
                Err(err) => {
                    tracing::error!(
                        "try to get subdataflow {:?} execution states failed: {:?}",
                        entry.key(),
                        err
                    );
                    states.set_status(DataflowStatus::Running);
                    states.subdataflow_infos.push(SubdataflowInfo {
                        execution_id: Some(entry.key().clone()),
                        executors_info: Default::default(),
                    })
                }
            }
        }

        states
    }
}

#[derive(Debug)]
pub(crate) enum TaskExecutionException {
    SubdataflowErrors(Vec<SubdataflowError>),
}

impl TaskExecutionException {
    pub(crate) fn to_tonic_status(&self) -> tonic::Status {
        todo!()
    }
}

use std::collections::BTreeMap;

use common::{net::cluster::Cluster, types::ExecutorId};
use proto::common::DataflowStatus;
use tokio::task::JoinHandle;
use tonic::async_trait;

use crate::executions::{
    SubdataflowDeploymentPlan, SubdataflowExecution, TaskDeploymentException, VertexRemoteExecution,
};

#[derive(Default)]
pub struct Scheduler {
    handlers: Vec<JoinHandle<()>>,
}

impl Scheduler {
    pub(crate) async fn terminate_dataflow(
        &mut self,
        cluster: &mut Cluster,
    ) -> Result<DataflowStatus, TaskExecutionException> {
        
    }

    pub(crate) async fn execute_all<I: Iterator<Item = SubdataflowDeploymentPlan>>(
        &mut self,
        cluster: &mut Cluster,
        plan_iter: I,
    ) -> Result<(), TaskDeploymentException> {
        let execution_futures = plan_iter.map(|plan| plan.deploy(cluster));
        let mut result = vec![];

        for future in execution_futures {
            match future.await {
                Ok(execution) => {
                    let handler = tokio::spawn(execution);
                    result.push(handler);
                }
                Err(err) => return Err(err),
            }
        }
        self.handlers = result;

        Ok(())
    }
}

pub(crate) enum TaskExecutionException {}

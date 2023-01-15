use std::{collections::BTreeMap, future::Future};

use common::{
    net::{cluster::Cluster, to_host_addr, PersistableHostAddr},
    types::ExecutorId,
};
use proto::{
    apiserver::CreateResourceRequest,
    common::{Dataflow, OperatorInfo, ResourceId},
    worker::CreateSubDataflowRequest,
    worker_gateway::SafeTaskManagerRpcGateway,
};

pub struct VertexRemoteExecution {
    executor_id: ExecutorId,
    operator: OperatorInfo,
}

impl VertexRemoteExecution {
    pub(crate) fn new(cluster: &mut Cluster, operator: &OperatorInfo) -> Self {}

    pub(crate) fn deploy(&mut self) -> Result<(), TaskDeploymentException> {
        todo!()
    }

    pub(crate) fn get_executor_id(&self) -> ExecutorId {
        self.executor_id
    }
}

/// [`SubdataflowDeploymentPlan`] represents a plan description for a subdataflow [`Dataflow`].
///
pub(crate) struct SubdataflowDeploymentPlan {
    subdataflow: Dataflow,
    addr: PersistableHostAddr,
}
impl SubdataflowDeploymentPlan {
    pub(crate) fn new(
        job_id: &ResourceId,
        subdataflow: (&PersistableHostAddr, &Dataflow),
        job_master_location: &PersistableHostAddr,
    ) -> Self {
        todo!()
    }

    pub(crate) async fn deploy(
        &self,
        cluster: &mut Cluster,
    ) -> Result<SubdataflowExecution, TaskDeploymentException> {
        match cluster.get_task_manager_gateway(&self.addr) {
            Some(gateway) => {
                let req = CreateSubDataflowRequest {
                    job_id: Some(self.subdataflow.get_job_id()),
                    dataflow: Some(self.subdataflow.clone()),
                };
                match gateway.create_sub_dataflow(req).await {
                    Ok(resp) => Ok(SubdataflowExecution {
                        job_id: self.subdataflow.get_job_id(),
                        worker_addr: self.addr.clone(),
                        gateway: gateway.clone(),
                    }),
                    Err(err) => {}
                }
            }
            None => todo!(),
        }
    }
}

pub(crate) enum TaskDeploymentException {}

pub(crate) struct SubdataflowExecution {
    job_id: ResourceId,
    worker_addr: PersistableHostAddr,
    gateway: SafeTaskManagerRpcGateway,
}

impl Future for SubdataflowExecution {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

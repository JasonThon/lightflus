use std::collections::BTreeMap;

use common::net::{
    cluster::{self, ClusterBuilder},
    AckResponderBuilder, HeartbeatBuilder, PersistableHostAddr,
};
use mockall_double::double;
use proto::common::{Ack, Dataflow, DataflowStatus, Heartbeat, ResourceId};

#[double]
use crate::scheduler::Scheduler;
use crate::{
    executions::{SubdataflowDeploymentPlan, TaskDeploymentException},
    storage::{DataflowStorage, DataflowStorageBuilder},
};

/// [`JobManager`] is responsible for
/// - monitor job's status
/// - terminate job
/// - checkpoint management
/// - recover a task from checkpoint
pub(crate) struct JobManager {
    dataflow: Dataflow,
    job_id: ResourceId,
    scheduler: Scheduler,
    location: PersistableHostAddr,
    storage: Box<dyn DataflowStorage>,
}
impl JobManager {
    pub(crate) fn new(
        location: &PersistableHostAddr,
        dataflow: Dataflow,
        storage: &DataflowStorageBuilder,
    ) -> Self {
        let job_id = dataflow.get_job_id();
        Self {
            dataflow,
            job_id,
            scheduler: Scheduler::new(),
            location: location.clone(),
            storage: storage.build(),
        }
    }

    /// Once a dataflow is deployed, JobManager will receive the event of state transition of each subdataflow from TaskManager.
    async fn deploy_dataflow(
        &mut self,
        cluster: &cluster::Cluster,
        heartbeat_builder: &HeartbeatBuilder,
        ack_builder: &AckResponderBuilder,
    ) -> Result<(), TaskDeploymentException> {
        let _ = self.storage.save(&self.dataflow);
        cluster.partition_dataflow(&mut self.dataflow);

        let subdataflow = cluster.split_into_subdataflow(&self.dataflow);
        let mut ack_builder = ack_builder.clone();
        ack_builder.nodes = vec![self.location.clone()];
        let executions = subdataflow.iter().map(|pair| {
            let plan = SubdataflowDeploymentPlan::new(
                pair,
                &self.job_id,
                cluster.get_node(pair.0),
                &ack_builder,
            );
            plan
        });

        for execution in executions {
            match self.scheduler.execute(execution, heartbeat_builder).await {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    async fn terminate_dataflow(&mut self) -> Result<DataflowStatus, tonic::Status> {
        self.scheduler
            .terminate_dataflow()
            .await
            .map_err(|err| err.to_tonic_status())
    }

    async fn update_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        for execution_id in heartbeat.execution_id.as_ref().iter() {
            match self.scheduler.get_execution_mut(&(*execution_id).into()) {
                Some(execution) => execution.update_heartbeat_status(heartbeat).await,
                None => {}
            }
        }
    }

    fn ack_from_execution(&mut self, ack: &Ack) {
        for execution_id in ack.execution_id.as_ref().iter() {
            match self.scheduler.get_execution_mut(&(*execution_id).into()) {
                Some(execution) => execution.ack(ack),
                None => {}
            }
        }
    }
}

/// [`Dispatcher`] is responsible for
/// - job submission
/// - dataflow persistance
/// - spawning job manager to manager each job's status
/// - job recovery
/// - heartbeat of remote cluster
pub(crate) struct Dispatcher {
    /// # TODO
    ///
    /// Change [`BTreeMap`] to an implementation of [`std::collections::HashMap`] to improve the request throughput
    managers: BTreeMap<ResourceId, JobManager>,
    cluster: cluster::Cluster,
    location: PersistableHostAddr,
    heartbeat: HeartbeatBuilder,
    ack: AckResponderBuilder,
    storage: DataflowStorageBuilder,
}

impl Dispatcher {
    pub fn new(
        cluster_builder: &ClusterBuilder,
        storage_builder: &DataflowStorageBuilder,
        heartbeat_builder: &HeartbeatBuilder,
        ack_builder: &AckResponderBuilder,
        port: usize,
    ) -> Self {
        let cluster = cluster_builder.build();
        Self {
            managers: Default::default(),
            cluster,
            location: PersistableHostAddr::local(port),
            heartbeat: heartbeat_builder.clone(),
            ack: ack_builder.clone(),
            storage: storage_builder.clone(),
        }
    }

    pub(crate) async fn create_dataflow(
        &self,
        dataflow: Dataflow,
    ) -> Result<(), DispatcherException> {
        let mut job_manager = JobManager::new(&self.location, dataflow, &self.storage);
        let result = job_manager
            .deploy_dataflow(&self.cluster, &self.heartbeat, &self.ack)
            .await
            .map_err(|err| DispatcherException::DeploymentError(err));

        result
    }

    pub(crate) async fn terminate_dataflow(
        &mut self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, DispatcherException> {
        match self.managers.get_mut(job_id) {
            Some(manager) => match manager.terminate_dataflow().await {
                Ok(status) => match &status {
                    DataflowStatus::Initialized => {
                        Err(DispatcherException::UnexpectedDataflowStatus(status))
                    }
                    DataflowStatus::Running => {
                        Err(DispatcherException::UnexpectedDataflowStatus(status))
                    }
                    DataflowStatus::Closing => Ok(status),
                    DataflowStatus::Closed => {
                        let _ = self.managers.remove(job_id);
                        Ok(status)
                    }
                },
                Err(err) => Err(DispatcherException::Tonic(err)),
            },
            None => Ok(DataflowStatus::Closed),
        }
    }

    pub(crate) fn get_dataflow(&self, job_id: &ResourceId) -> Option<Dataflow> {
        todo!()
    }

    pub(crate) async fn update_task_manager_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        match heartbeat.execution_id.as_ref() {
            Some(execution_id) => {
                for resource_id in execution_id.job_id.as_ref().iter() {
                    match self.managers.get_mut(*resource_id) {
                        Some(manager) => manager.update_heartbeat_status(heartbeat).await,
                        None => {}
                    }
                }
            }
            None => {}
        }
    }

    pub(crate) fn ack_from_task_manager(&mut self, ack: Ack) {
        match ack.execution_id.as_ref() {
            Some(execution_id) => {
                for resource_id in execution_id.job_id.as_ref().iter() {
                    match self.managers.get_mut(*resource_id) {
                        Some(manager) => manager.ack_from_execution(&ack),
                        None => {}
                    }
                }
            }
            None => {}
        }
    }
}

pub(crate) enum DispatcherException {
    Tonic(tonic::Status),
    DeploymentError(TaskDeploymentException),
    UnexpectedDataflowStatus(DataflowStatus),
}

impl DispatcherException {
    pub(crate) fn to_tonic_status(&self) -> tonic::Status {
        match self {
            DispatcherException::Tonic(status) => status.clone(),
            DispatcherException::UnexpectedDataflowStatus(status) => {
                tonic::Status::internal(format!("unexpected dataflow status {:?}", status))
            }
            DispatcherException::DeploymentError(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use proto::common::DataflowMeta;

    use super::*;

    #[tokio::test]
    async fn test_job_manager_deploy_success() {
        let mut mock_scheduler = Scheduler::default();

        mock_scheduler
            .expect_execute()
            .times(1)
            .returning(|_, _| Ok(()));

        let storage_builder = DataflowStorageBuilder::Memory;

        let mut manager = JobManager {
            dataflow: Dataflow {
                job_id: Default::default(),
                meta: vec![DataflowMeta {
                    center: 0,
                    neighbors: vec![],
                }],
                nodes: HashMap::from_iter([(0, Default::default())].into_iter()),
                execution_id: Default::default(),
            },
            job_id: Default::default(),
            scheduler: mock_scheduler,
            location: Default::default(),
            storage: storage_builder.build(),
        };
        let builder = ClusterBuilder {
            nodes: vec![],
            rpc_timeout: 3,
            connect_timeout: 3,
        };
        let c = builder.build();
        let ref heartbeat_builder = HeartbeatBuilder {
            nodes: vec![],
            period: 3,
            connect_timeout: 3,
            rpc_timeout: 3,
        };
        let ref ack_builder = AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            nodes: vec![],
            connect_timeout: 3,
            rpc_timeout: 3,
        };

        let result = manager
            .deploy_dataflow(&c, heartbeat_builder, ack_builder)
            .await;
        assert!(result.is_ok())
    }
}

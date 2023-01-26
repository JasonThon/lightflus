use std::collections::BTreeMap;

use common::{
    net::{cluster, AckResponderBuilder, HeartbeatBuilder, PersistableHostAddr},
    types::HashedResourceId,
    ExecutionID,
};
use mockall_double::double;
use proto::common::{Ack, Dataflow, DataflowStatus, Heartbeat, HostAddr, ResourceId};

#[double]
use crate::scheduler::Scheduler;
use crate::{
    config::CoordinatorConfig,
    executions::{SubdataflowDeploymentPlan, TaskDeploymentException},
    storage::DataflowStorageImpl,
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
}
impl JobManager {
    pub(crate) fn new(location: &PersistableHostAddr, dataflow: &Dataflow) -> Self {
        Self {
            dataflow: dataflow.clone(),
            job_id: dataflow.get_job_id(),
            scheduler: Scheduler::new(),
            location: location.clone(),
        }
    }

    /// Once a dataflow is deployed, JobManager will receive the event of state transition of each subdataflow from TaskManager.
    async fn deploy_dataflow(
        &mut self,
        cluster: &cluster::Cluster,
        heartbeat_builder: &HeartbeatBuilder,
        ack_builder: &AckResponderBuilder,
    ) -> Result<(), TaskDeploymentException> {
        let subdataflow = cluster.split_into_subdataflow(&self.dataflow);
        let mut execution_id = 0;
        let mut ack_builder = ack_builder.clone();
        ack_builder.nodes = vec![self.location.clone()];
        let executions = subdataflow.iter().map(|pair| {
            let plan = SubdataflowDeploymentPlan::new(
                pair,
                ExecutionID(self.job_id.clone(), execution_id),
                cluster.get_node(pair.0),
                cluster.get_connect_timeout(),
                cluster.get_rpc_timeout(),
                &ack_builder,
            );
            execution_id += 1;
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

    async fn terminate_dataflow(
        &mut self,
        cluster: &mut cluster::Cluster,
    ) -> Result<DataflowStatus, tonic::Status> {
        self.scheduler
            .terminate_dataflow(cluster)
            .await
            .map_err(|err| err.to_tonic_status())
    }

    async fn update_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        for execution_id in heartbeat.execution_id.as_ref().iter() {
            match self.scheduler.get_execution_mut((*execution_id).into()) {
                Some(execution) => execution.update_heartbeat_status(heartbeat).await,
                None => {}
            }
        }
    }

    fn ack_from_execution(&mut self, ack: &Ack) {
        for execution_id in ack.execution_id.as_ref().iter() {
            match self.scheduler.get_execution_mut((*execution_id).into()) {
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
    managers: BTreeMap<HashedResourceId, JobManager>,
    cluster: cluster::Cluster,
    dataflow_storage: DataflowStorageImpl,
    location: PersistableHostAddr,
    heartbeat: HeartbeatBuilder,
    ack: AckResponderBuilder,
}

impl Dispatcher {
    pub fn new(config: &CoordinatorConfig) -> Self {
        let dataflow_storage = config.storage.to_dataflow_storage();
        let mut cluster = cluster::Cluster::new(&config.cluster);
        cluster.set_rpc_timeout(config.rpc_timeout);
        cluster.set_connect_timeout(config.connect_timeout);
        Self {
            managers: Default::default(),
            cluster,
            dataflow_storage,
            location: PersistableHostAddr::local(config.port),
            heartbeat: config.heartbeat.clone(),
            ack: config.ack.clone(),
        }
    }

    pub(crate) async fn create_dataflow(
        &mut self,
        dataflow: &mut Dataflow,
    ) -> Result<(), DispatcherException> {
        match self.dataflow_storage.save(dataflow.clone()) {
            Err(err) => {
                return Err(DispatcherException::Tonic(tonic::Status::internal(
                    err.message,
                )))
            }
            _ => {
                self.cluster.partition_dataflow(dataflow);
                let mut job_manager = JobManager::new(&self.location, dataflow);
                let result = job_manager
                    .deploy_dataflow(&self.cluster, &self.heartbeat, &self.ack)
                    .await
                    .map_err(|err| DispatcherException::DeploymentError(err));

                result
            }
        }
    }

    pub(crate) async fn terminate_dataflow(
        &mut self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, DispatcherException> {
        if !self.dataflow_storage.may_exists(job_id) {
            Ok(DataflowStatus::Closed)
        } else {
            let hashed_job_id = &HashedResourceId::from(job_id);
            match self.managers.get_mut(hashed_job_id) {
                Some(manager) => match manager.terminate_dataflow(&mut self.cluster).await {
                    Ok(status) => match &status {
                        DataflowStatus::Initialized => {
                            Err(DispatcherException::UnexpectedDataflowStatus(status))
                        }
                        DataflowStatus::Running => {
                            Err(DispatcherException::UnexpectedDataflowStatus(status))
                        }
                        DataflowStatus::Closing => Ok(status),
                        DataflowStatus::Closed => {
                            let _ = self.managers.remove(hashed_job_id);
                            Ok(status)
                        }
                    },
                    Err(err) => Err(DispatcherException::Tonic(err)),
                },
                None => Ok(DataflowStatus::Closed),
            }
        }
    }

    pub(crate) fn get_dataflow(&self, job_id: &ResourceId) -> Option<Dataflow> {
        self.dataflow_storage.get(job_id)
    }

    pub(crate) async fn update_task_manager_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        match heartbeat.execution_id.as_ref() {
            Some(execution_id) => {
                for resource_id in execution_id.job_id.as_ref().iter() {
                    match self.managers.get_mut(&(*resource_id).into()) {
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
                    match self.managers.get_mut(&(*resource_id).into()) {
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

        let mut manager = JobManager {
            dataflow: Dataflow {
                job_id: Default::default(),
                meta: vec![DataflowMeta {
                    center: 0,
                    neighbors: vec![],
                }],
                nodes: HashMap::from_iter([(0, Default::default())].into_iter()),
            },
            job_id: Default::default(),
            scheduler: mock_scheduler,
            location: Default::default(),
        };
        let ref c = cluster::Cluster::new(&vec![]);
        let ref heartbeat_builder = HeartbeatBuilder {
            node_addrs: vec![],
            period: 3,
            connection_timeout: 3,
            rpc_timeout: 3,
        };
        let ref ack_builder = AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            nodes: vec![],
        };

        let result = manager
            .deploy_dataflow(c, heartbeat_builder, ack_builder)
            .await;
        assert!(result.is_ok())
    }
}

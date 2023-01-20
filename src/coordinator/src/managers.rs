use std::collections::BTreeMap;

use common::{
    net::{cluster, AckResponderBuilder, HeartbeatBuilder, PersistableHostAddr},
    types::HashedResourceId,
    ExecutionID,
};
use proto::common::{Dataflow, DataflowStatus, Heartbeat, HostAddr, ResourceId};

use crate::{
    config::CoordinatorConfig,
    executions::{SubdataflowDeploymentPlan, TaskDeploymentException},
    scheduler::Scheduler,
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
            scheduler: Scheduler::default(),
            location: location.clone(),
        }
    }

    /// Once a dataflow is deployed, JobManager will receive the event of state transition of each subdataflow from TaskManager.
    async fn deploy_dataflow(
        &mut self,
        cluster: &mut cluster::Cluster,
        heartbeat_builder: &HeartbeatBuilder,
        ack_builder: &mut AckResponderBuilder,
    ) -> Result<(), TaskDeploymentException> {
        let subdataflow = cluster.split_into_subdataflow(&self.dataflow);
        let mut execution_id = 0;
        ack_builder.nodes = vec![self.location.clone()];
        let executions = subdataflow.iter().map(|pair| {
            let plan = SubdataflowDeploymentPlan::new(
                pair,
                ExecutionID(self.job_id.clone(), execution_id),
                cluster.get_node(pair.0),
                ack_builder,
            );
            execution_id += 1;
            plan
        });

        self.scheduler
            .execute_all(executions, heartbeat_builder)
            .await
    }

    async fn terminate_dataflow(
        &mut self,
        cluster: &mut cluster::Cluster,
    ) -> Result<DataflowStatus, tonic::Status> {
        todo!()
    }

    async fn update_heartbeat_status(&mut self, heartbeat: &Heartbeat) {
        for execution_id in heartbeat.execution_id.as_ref().iter() {
            match self.scheduler.get_execution_mut((*execution_id).into()) {
                Some(execution) => execution.update_heartbeat_status(heartbeat).await,
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
        Self {
            managers: Default::default(),
            cluster: cluster::Cluster::new(&config.cluster),
            dataflow_storage,
            location: PersistableHostAddr::local(config.port),
            heartbeat: HeartbeatBuilder {
                period: config.heartbeat.period,
                node_addrs: config
                    .cluster
                    .iter()
                    .map(|node_conf| {
                        (
                            HostAddr {
                                host: node_conf.host.clone(),
                                port: node_conf.port as u32,
                            },
                            config.heartbeat.connection_timeout,
                        )
                    })
                    .collect(),
            },
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
                    .deploy_dataflow(&mut self.cluster, &self.heartbeat, &mut self.ack)
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

use std::collections::BTreeMap;

use common::{
    net::{cluster, hostname, PersistableHostAddr},
    types::HashedResourceId,
};
use proto::common::{Dataflow, DataflowStatus, ResourceId};

use crate::{
    coord::CoordinatorConfig,
    executions::{SubdataflowDeploymentPlan, TaskDeploymentException, VertexRemoteExecution},
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
    pub(crate) fn new(location: &PersistableHostAddr) -> Self {
        Self {
            dataflow: Default::default(),
            job_id: Default::default(),
            scheduler: Scheduler::default(),
            location: location.clone(),
        }
    }

    async fn create_dataflow(
        &mut self,
        cluster: &mut cluster::Cluster,
        dataflow: &mut Dataflow,
    ) -> Result<(), TaskDeploymentException> {
        cluster.partition_dataflow(dataflow);
        self.job_id = dataflow.get_job_id();
        self.dataflow = dataflow.clone();

        let subdataflow = cluster.split_into_subdataflow(dataflow);
        let executions = subdataflow
            .iter()
            .map(|pair| SubdataflowDeploymentPlan::new(&self.job_id, pair, &self.location));

        self.scheduler.execute_all(cluster, executions).await
    }

    async fn terminate_dataflow(
        &mut self,
        cluster: &mut cluster::Cluster,
    ) -> Result<DataflowStatus, tonic::Status> {
        self.scheduler.terminate_dataflow(cluster).await
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
}

impl Dispatcher {
    pub fn new(config: &CoordinatorConfig) -> Self {
        let dataflow_storage = config.storage.to_dataflow_storage();
        Self {
            managers: Default::default(),
            cluster: cluster::Cluster::new(&config.cluster),
            dataflow_storage,
            location: PersistableHostAddr::local(config.port),
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
                let mut job_manager = JobManager::new(&self.location);
                let result = job_manager
                    .create_dataflow(&mut self.cluster, dataflow)
                    .await
                    .map_err(|err| DispatcherException::DeploymentError(err));

                let job_id = HashedResourceId::from(dataflow.get_job_id());

                self.managers.insert(job_id, job_manager);
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

    pub(crate) async fn probe_cluster_state(&mut self) {
        self.cluster.probe_state().await
    }
}

pub enum DispatcherException {
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
        }
    }
}

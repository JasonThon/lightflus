use common::net::{
    cluster::{self, ClusterBuilder},
    local, AckResponderBuilder, HeartbeatBuilder,
};
use crossbeam_skiplist::SkipMap;
use proto::common::{Ack, Dataflow, DataflowStatus, Heartbeat, HostAddr, ResourceId};

use super::{
    executions::{SubdataflowDeploymentPlan, TaskDeploymentException},
    scheduler::Scheduler,
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
    location: HostAddr,
    storage: Box<dyn DataflowStorage>,
}
impl JobManager {
    pub(crate) fn new(
        location: &HostAddr,
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

        let mut subdataflow = cluster.split_into_subdataflow(&self.dataflow);
        let mut ack_builder = ack_builder.clone();
        ack_builder.nodes = vec![self.location.clone()];
        let executions = subdataflow.iter_mut().map(|pair| {
            let host_addr = pair.0;
            let plan = SubdataflowDeploymentPlan::new(
                pair,
                &self.job_id,
                cluster.get_node(host_addr),
                &ack_builder,
                heartbeat_builder,
            );
            plan
        });

        for execution in executions {
            match self.scheduler.execute(execution).await {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }

    async fn terminate_dataflow(&self) -> Result<DataflowStatus, tonic::Status> {
        self.scheduler
            .terminate_dataflow()
            .await
            .map_err(|err| err.to_tonic_status())
    }

    async fn update_heartbeat_status(&self, heartbeat: &Heartbeat) {
        for execution_id in heartbeat.subdataflow_id.as_ref().iter() {
            self.scheduler.receive_heartbeat(heartbeat).await;
        }
    }

    fn ack_from_execution(&self, ack: &Ack) {
        for execution_id in ack.execution_id.as_ref().iter() {
            self.scheduler.ack(ack);
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
    managers: SkipMap<ResourceId, JobManager>,
    cluster: cluster::Cluster,
    location: HostAddr,
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
            location: local(port),
            heartbeat: heartbeat_builder.clone(),
            ack: ack_builder.clone(),
            storage: storage_builder.clone(),
        }
    }

    pub(crate) async fn create_dataflow(
        &self,
        dataflow: Dataflow,
    ) -> Result<(), DispatcherException> {
        let job_id = dataflow.get_job_id();
        let mut job_manager = JobManager::new(&self.location, dataflow, &self.storage);
        let result = job_manager
            .deploy_dataflow(&self.cluster, &self.heartbeat, &self.ack)
            .await
            .map_err(|err| DispatcherException::DeploymentError(err));
        self.managers.insert(job_id, job_manager);

        result
    }

    pub(crate) async fn terminate_dataflow(
        &self,
        job_id: &ResourceId,
    ) -> Result<DataflowStatus, DispatcherException> {
        match self.managers.get(job_id) {
            Some(manager) => match manager.value().terminate_dataflow().await {
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

    pub(crate) async fn update_task_manager_heartbeat_status(&self, heartbeat: &Heartbeat) {
        match heartbeat
            .subdataflow_id
            .as_ref()
            .and_then(|execution_id| execution_id.job_id.as_ref())
            .and_then(|resource_id| self.managers.get(resource_id))
        {
            Some(entry) => entry.value().update_heartbeat_status(heartbeat).await,
            None => {}
        }
    }

    pub(crate) async fn ack_from_task_manager(&self, ack: Ack) {
        match ack
            .execution_id
            .as_ref()
            .and_then(|execution_id| execution_id.job_id.as_ref())
            .and_then(|resource_id| self.managers.get(resource_id))
        {
            Some(manager) => manager.value().ack_from_execution(&ack),
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

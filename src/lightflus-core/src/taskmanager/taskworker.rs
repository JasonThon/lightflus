use std::collections::BTreeMap;
use std::collections::HashMap;

use common::event::LocalEvent;
use common::types::ExecutorId;
use common::utils::is_remote_operator;
use proto::common::Ack;
use proto::common::Dataflow;
use proto::common::Heartbeat;
use proto::common::KeyedDataEvent;

use proto::common::KeyedEventSet;
use proto::common::NodeType;

use proto::common::SubDataflowId;
use proto::common::SubdataflowInfo;
use proto::taskmanager::SendEventToOperatorStatusEnum;

use stream::connector::SinkImpl;
use stream::task::EdgeBuilder;

use stream::task::Task;

use crate::errors::taskmanager::TaskWorkerError;

#[derive(Default)]
pub struct TaskWorker {
    tasks: HashMap<ExecutorId, Task>,
    subdataflow_id: SubDataflowId,
}

pub(crate) struct TaskWorkerBuilder<'a> {
    dataflow: &'a Dataflow,
}

impl<'a> TaskWorkerBuilder<'a> {
    pub(crate) fn new(dataflow: &'a Dataflow) -> Self {
        Self { dataflow }
    }

    pub(crate) async fn build(&self) -> Result<TaskWorker, TaskWorkerError> {
        self.dataflow
            .validate()
            .map(|_| {
                let mut raw_tasks = BTreeMap::new();
                let mut edge_builders = BTreeMap::new();

                let mut worker = TaskWorker::default();
                worker.subdataflow_id = self
                    .dataflow
                    .get_execution_id_ref()
                    .map(|id| id.clone())
                    .unwrap_or_default();

                let job_id = self.dataflow.job_id.as_ref().unwrap();
                let info_set = &self.dataflow.nodes;
                self.dataflow.meta.iter().for_each(|meta| {
                    let info = info_set.get(&meta.center).unwrap();
                    let task = Task::new(job_id, &meta);
                    edge_builders.insert(meta.center, EdgeBuilder::local(info));

                    meta.neighbors.iter().for_each(|neighbor_id| {
                        let neighbor_info = info_set.get(neighbor_id).unwrap();
                        if is_remote_operator(neighbor_info)
                            && !edge_builders.contains_key(neighbor_id)
                        {
                            edge_builders.insert(*neighbor_id, EdgeBuilder::remote(neighbor_info));
                        } else if !edge_builders.contains_key(neighbor_id) {
                            edge_builders.insert(*neighbor_id, EdgeBuilder::local(info));
                        }
                    });

                    raw_tasks.insert(meta.center, task);
                });

                worker.tasks = raw_tasks
                    .into_iter()
                    .map(|(executor_id, mut task)| {
                        let operator_info = info_set.get(&executor_id).unwrap();

                        let mut executor = task.create_stream_executor(operator_info);
                        task.get_downstream_id_iter().for_each(|dowstream_id| {
                            edge_builders.get(dowstream_id).iter().for_each(|builder| {
                                let out_edge = (*builder).build_out_edge();
                                executor.add_out_edge(*dowstream_id, out_edge);
                            });
                        });

                        // if operator is not Source, it should create an out-edge for [`TaskWorker`] to send operator
                        if !operator_info.has_source() {
                            let builder = edge_builders.remove(&executor_id).unwrap();
                            task.set_in_edge(builder.build_out_edge());
                            executor.set_in_edge(builder.build_in_edge())
                        }

                        if operator_info.has_sink() {
                            executor.add_external_sink(SinkImpl::from((job_id, operator_info)))
                        }

                        task.start(executor);

                        (executor_id, task)
                    })
                    .collect();

                worker
            })
            .map_err(|err| TaskWorkerError::DataflowValidateError(err))
    }
}

impl TaskWorker {
    #[inline]
    pub async fn send_event_to_operator(
        &self,
        event: KeyedDataEvent,
    ) -> Result<SendEventToOperatorStatusEnum, TaskWorkerError> {
        let executor_id = event.to_operator_id;
        match self.tasks.get(&executor_id) {
            Some(task) => task
                .send_event_to_operator(LocalEvent::KeyedDataStreamEvent(event))
                .await
                .map(|_| SendEventToOperatorStatusEnum::Done)
                .map_err(|err| TaskWorkerError::EventSendFailure(err.to_string())),
            None => Ok(SendEventToOperatorStatusEnum::Done),
        }
    }

    #[inline]
    pub fn receive_heartbeat(&self, heartbeat: &Heartbeat) {
        match heartbeat.node_type() {
            NodeType::JobManager => match self.tasks.get(&heartbeat.task_id) {
                Some(task) => task.receive_heartbeat(heartbeat),
                None => {}
            },
            _ => {}
        }
    }

    #[inline]
    pub fn receive_ack(&self, ack: &Ack) {
        match ack.get_execution_id() {
            Some(execution_id) => match self.tasks.get(&execution_id.sub_id) {
                Some(task) => task.receive_ack(ack),
                None => {}
            },
            None => {}
        }
    }

    #[inline]
    pub async fn batch_send_event_to_operator(
        &self,
        event_set: KeyedEventSet,
    ) -> Result<SendEventToOperatorStatusEnum, TaskWorkerError> {
        match self.tasks.get(&event_set.to_operator_id) {
            Some(task) => task
                .batch_send_event_to_operator(event_set)
                .await
                .map(|_| SendEventToOperatorStatusEnum::Done)
                .map_err(|err| TaskWorkerError::EventSendFailure(err.to_string())),
            None => Ok(SendEventToOperatorStatusEnum::Done),
        }
    }

    pub async fn get_state(&self) -> SubdataflowInfo {
        let mut info = SubdataflowInfo {
            execution_id: Some(self.subdataflow_id.clone()),
            executors_info: Default::default(),
        };

        for (executor_id, task) in &self.tasks {
            info.executors_info
                .insert(*executor_id, task.get_state().await);
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use proto::common::{Dataflow, OperatorInfo, ResourceId, SubDataflowId};

    use super::TaskWorkerBuilder;

    #[tokio::test]
    async fn test_task_worker_build() {
        let dataflow = Dataflow {
            job_id: Some(ResourceId {
                resource_id: "resource_id".to_string(),
                namespace_id: "namespace_id".to_string(),
            }),
            meta: vec![],
            nodes: HashMap::from_iter(vec![(0, OperatorInfo::default())].into_iter()),
            execution_id: Some(SubDataflowId {
                job_id: Some(ResourceId {
                    resource_id: "resource_id".to_string(),
                    namespace_id: "namespace_id".to_string(),
                }),
                sub_id: 0,
            }),
        };
        let builder = TaskWorkerBuilder::new(&dataflow);
        let result = builder.build().await;
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn test_edge_builder_build_out_edge() {}
}

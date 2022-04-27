use std::collections;

use crate::{err, event, runtime, types};
use crate::err::Error;

pub struct TaskWorker {
    job_pool: collections::HashMap<types::JobID, runtime::Graph>,
}

struct TaskWorkerBuilder {}

impl actix::Actor for TaskWorker {
    type Context = actix::Context<Self>;
}

impl actix::Handler<event::GraphEvent> for TaskWorker {
    type Result = ();

    fn handle(&mut self, event: event::GraphEvent, ctx: &mut Self::Context) -> Self::Result {
        match event {
            event::GraphEvent::ExecutionGraphSubmit {
                job_id, ops
            } => self.build_new_graph(job_id, runtime::to_execution_graph(ops)),

            event::GraphEvent::DataSourceEventSubmit(ope) => {
                match self.submit_datasource_event(ope) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("submit event failed: {:?}", err);
                    }
                }
            }
            event::GraphEvent::StopGraph { job_id } => {
                log::debug!("start stopping job {:?}", &job_id);
                match self.stop_job(&job_id) {
                    Ok(_) => log::debug!("stop job success"),
                    Err(err) => {
                        log::error!("stop job {:?} failed: {:?}",job_id , err);
                    }
                }
            }
            event::GraphEvent::FormulaOpEventSubmit {
                job_id, events, to
            } => {
                log::debug!("formula op events submitted. job id: {:?}, events: {:?}", &job_id, &events);
                match self.submit_formula_op_events(job_id, to, events) {
                    Ok(_) => log::debug!("formula op events submit success"),
                    Err(err) => {
                        log::error!("formula op events failed: {:?}", err);
                    }
                }
            }
        }
    }
}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            job_pool: Default::default(),
        }
    }

    pub fn submit_datasource_event(&mut self, event: event::DataSourceEvent) -> Result<(), err::ExecutionException> {
        let ref job_id = event.job_id.clone();
        match self.job_pool
            .get(job_id) {
            Some(graph) => graph.try_recv(event)
                .map_err(|err| {
                    log::error!(
                        "Error when submit event. JobId {:?}, time: {:?}. error detail {}",
                        job_id,
                        std::time::SystemTime::now(),
                        err.to_string()
                    );
                    err
                }),
            None => Ok(())
        }
    }

    pub fn build_new_graph(&mut self, job_id: types::JobID, ops: runtime::Graph) {
        self.job_pool.insert(job_id.clone(), ops);
        self.job_pool.get_mut(&job_id)
            .unwrap()
            .build_dag(job_id);
    }

    pub fn stop_job(&mut self, job_id: &types::JobID) -> Result<(), err::TaskWorkerError> {
        match self.job_pool.get(job_id) {
            Some(graph) => graph.stop()
                .map(|_| {
                    self.job_pool.remove(job_id);
                })
                .map_err(|err| err.into()),
            None => Ok(())
        }
    }

    pub fn submit_formula_op_events(&mut self,
                                    job_id: types::JobID,
                                    to: u64,
                                    events: Vec<event::FormulaOpEvent>) -> Result<(), err::TaskWorkerError> {
        match self.job_pool.get_mut(&job_id) {
            Some(graph) => graph.try_send_formula_op_events(to, events)
                .map_err(|err| err.into()),
            None => Ok(())
        }
    }
}

impl TaskWorkerBuilder {
    pub(crate) fn build(&self) -> TaskWorker {
        TaskWorker::new()
    }


    pub(crate) fn new() -> Self {
        TaskWorkerBuilder {}
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct TaskWorkerConfig {
    pub port: usize,
}

pub fn new_worker() -> TaskWorker {
    TaskWorkerBuilder::new()
        .build()
}

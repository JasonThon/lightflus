use std::borrow::{Borrow, BorrowMut};
use std::cell;
use std::collections;
use std::sync;
use actix_web::dev::Service;

use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::{SendError, TryRecvError};

use crate::{err, event, runtime, types};
use crate::runtime::execution;
use crate::types::formula;
use crate::event::FormulaOpEvent;
use crate::types::JobID;

pub struct TaskWorker {
    job_pool: cell::RefCell<collections::HashMap<types::JobID, runtime::Graph>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum TaskWorkerError {
    ChannelDisconnected,
    ChannelEmpty,
}

impl From<mpsc::error::SendError<types::JobID>> for TaskWorkerError {
    fn from(err: SendError<JobID>) -> Self {
        todo!()
    }
}

impl From<mpsc::error::TryRecvError> for TaskWorkerError {
    fn from(err: TryRecvError) -> Self {
        match err {
            TryRecvError::Empty => TaskWorkerError::ChannelEmpty,
            TryRecvError::Closed => TaskWorkerError::ChannelDisconnected
        }
    }
}

impl From<err::ExecutionException> for TaskWorkerError {
    fn from(_: err::ExecutionException) -> Self {
        todo!()
    }
}

struct TaskWorkerBuilder {}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            job_pool: Default::default(),
        }
    }

    pub fn submit_event(&self, event: event::FormulaOpEvent) -> Result<(), TaskWorkerError> {
        let result = cell::RefCell::new(Ok(()));

        cell::RefMut::map(
            self.job_pool.borrow_mut(),
            |pool| {
                match pool.get_mut(&event.job_id) {
                    Some(graph) => {
                        result.replace(graph.try_recv(event));
                    }
                    _ => {}
                }

                pool
            },
        );


        result.take()
            .map_err(|err| TaskWorkerError::from(err))
    }

    pub fn build_new_graph(&self, job_id: types::JobID, ops: runtime::Graph) {
        cell::RefMut::map(
            self.job_pool.borrow_mut(),
            |map| {
                map.insert(job_id.clone(), ops);
                map.get_mut(&job_id)
                    .unwrap()
                    .build_dag(job_id);

                map
            },
        );
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskWorkerConfig {
    pub port: usize,
}

pub fn new_worker() -> TaskWorker {
    TaskWorkerBuilder::new()
        .build()
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GraphEvent {
    ExecutionGraphSubmit {
        ops: runtime::Graph,
        job_id: types::JobID,
    },
    NodeEventSubmit(FormulaOpEvent),
}

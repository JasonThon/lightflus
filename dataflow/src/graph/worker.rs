use std::borrow::{Borrow, BorrowMut};
use std::cell;
use std::cell::{RefCell, RefMut};
use std::collections;
use std::sync;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::join;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::{SendError, TryRecvError};

use crate::runtime;
use crate::runtime::execution;
use crate::runtime::formula;
use crate::runtime::formula::FormulaOpEvent;
use crate::types;
use crate::types::JobID;

pub struct TaskWorker {
    job_pool: cell::RefCell<collections::HashMap<types::JobID, runtime::Graph>>,
    event_in_tx_map: cell::RefCell<collections::HashMap<types::JobID, mpsc::UnboundedSender<formula::FormulaOpEvent>>>,
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

struct TaskWorkerBuilder {}

impl TaskWorker {
    pub(crate) fn new() -> Self {
        TaskWorker {
            job_pool: Default::default(),
            event_in_tx_map: Default::default(),
        }
    }

    pub fn submit_event(&self, event: formula::FormulaOpEvent) -> Result<(), TaskWorkerError> {
        RefMut::map(
            self.event_in_tx_map.borrow_mut(),
            |tx| {
                if tx.contains_key(&event.job_id) {
                    tx.get_mut(&event.job_id)
                        .unwrap()
                        .send(event);
                }

                tx
            },
        );

        Ok(())
    }

    pub fn build_new_graph(&self, job_id: types::JobID, mut ops: runtime::Graph) {
        ops.build_dag();
        cell::RefMut::map(
            self.job_pool.borrow_mut(),
            |map| {
                ops.start();
                map.insert(job_id, ops);
                map
            }
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

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum GraphEvent {
    ExecutionGraphSubmit {
        ops: runtime::Graph,
        job_id: types::JobID,
    },
    NodeEventSubmit(runtime::formula::FormulaOpEvent),
}

pub fn new_worker() -> TaskWorker {
    TaskWorkerBuilder::new()
        .build()
}
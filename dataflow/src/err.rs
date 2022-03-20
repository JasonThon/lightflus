use std::fmt;

use tokio::sync::mpsc;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoordinatorException {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ErrorKind {}

impl From<ExecutionException> for CoordinatorException {
    fn from(err: ExecutionException) -> Self {
        todo!()
    }
}


pub(crate) struct ConnectionError {}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecutionException {
    pub kind: ErrorKind,
    pub msg: String,
}

impl From<actix::prelude::SendError<super::event::FormulaOpEvent>> for ExecutionException {
    fn from(err: actix::prelude::SendError<super::event::FormulaOpEvent>) -> Self {
        todo!()
    }
}

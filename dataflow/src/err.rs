use std::fmt;

use tokio::sync::mpsc;

use crate::runtime;
use crate::runtime::execution;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CoordinatorException {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ErrorKind {}

impl From<runtime::ExecutionException> for CoordinatorException {
    fn from(err: runtime::ExecutionException) -> Self {
        todo!()
    }
}
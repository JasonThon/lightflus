use std::fmt;
use tokio::sync::mpsc;

use core::event;

pub enum TableError {
    EventSendError(mpsc::error::SendError<event::Event>),
    InvalidJson(serde_json::error::Error),
    InsertRecordFailed(mongodb::error::Error),
    FailToFindDocument(mongodb::error::Error),
    TableNotExist(String)
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
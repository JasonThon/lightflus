use serde::{Deserialize, Serialize};
use serde_json::Error;
use core::http;
use crate::core::domain;

#[derive(Clone, Serialize, Deserialize)]
pub enum TableAction {
    Insert {
        data: Vec<u8>,
        id: String,
    },
    Create {
        name: String,
        _type: domain::TableType
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TableCommand {
    action: TableAction,
}

impl TableCommand {
    pub fn action(&self) -> &TableAction {
        &self.action
    }
}

#[actix_web::post("/tables")]
pub async fn table_api(cmd: actix_web::web::Json<TableCommand>,
                       worker: actix_web::web::Data<super::core::svc::TableWorker>) -> actix_web::web::Json<http::Response> {
    let result = worker.process_cmd(cmd.0).await;
    match result {
        Ok(resp) => actix_web::web::Json(resp),
        Err(err) => panic!("{}", err)
    }
}
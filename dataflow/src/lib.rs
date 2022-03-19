use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};

use actix_web::web;

#[cfg(feature = "events")]
pub mod event;
#[cfg(feature = "errors")]
pub mod err;
#[cfg(feature = "coord")]
pub mod coord;
#[cfg(feature = "source")]
pub mod source;
#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "coord")]
pub mod cluster;
#[cfg(feature = "types")]
pub mod types;
#[cfg(feature = "graph")]
pub mod graph;

type TableResult = actix_web::Result<String>;

#[actix_web::post("/update_table")]
#[cfg(feature = "coord")]
pub async fn table_api(event: web::Json<event::TableEvent>,
                       coordinator: web::Data<coord::Coordinator>) -> String {
    match event.0.action() {
        event::TableAction::FormulaUpdate {
            table_id, header_id, graph
        } => {
            match coordinator
                .submit_job(table_id, header_id, graph) {
                Ok(()) => core::http::Response::ok().to_string(),
                Err(err) => "".to_string()
            }
        }
        _ => core::http::Response::ok().to_string()
    }
}

#[actix_web::get("/overview")]
#[cfg(feature = "coord")]
pub async fn overview() -> Result<actix_web::HttpResponse, actix_web::Error> {
    actix_web::HttpResponse::Ok()
        .body("ok")
        .await
}

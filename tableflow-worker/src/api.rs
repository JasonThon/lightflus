use std::collections;
use std::ops::Deref;
use std::sync::Arc;

use actix_web::web;

use dataflow::{graph, graph::worker, types};
use dataflow::graph::worker::TaskWorkerError;
use dataflow::runtime::formula::FormulaOpEvent;

type JoinHandlerMap = collections::HashMap<types::JobID, tokio::task::JoinHandle<()>>;

#[actix_web::post("/actionToEvents")]
pub async fn action_to_events(event: web::Json<graph::worker::GraphEvent>,
                              worker: web::Data<worker::TaskWorker>) -> Result<actix_web::HttpResponse, actix_web::Error> {
    match event.0 {
        graph::worker::GraphEvent::ExecutionGraphSubmit {
            job_id, ops
        } => {
            worker.build_new_graph(job_id, ops);
            Ok(actix_web::HttpResponse::Ok().finish())
        },
        graph::worker::GraphEvent::NodeEventSubmit(event) => {
            match worker.submit_event(event) {
                Ok(_) => actix_web::HttpResponse::Ok()
                    .body("ok")
                    .await,
                Err(err) => actix_web::HttpResponse::InternalServerError()
                    .body(serde_json::to_string(&err)?)
                    .await
            }
        }
    }
}

#[actix_web::get("/overview")]
pub async fn overview() -> Result<actix_web::HttpResponse, actix_web::Error> {
    actix_web::HttpResponse::Ok()
        .body("ok")
        .await
}
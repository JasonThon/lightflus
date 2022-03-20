use std::collections;

use actix_web;
use tokio::io::AsyncReadExt;

use dataflow::worker;
use dataflow::types;

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = worker::TaskWorkerConfig { port: 8000 };

    actix_web::HttpServer::new(||
        actix_web::App::new()
            .data(worker::new_worker())
            .service(actix_web::web::scope("/taskWorker")
                .service(api::action_to_events)
                .service(api::overview)
            ))
        .bind(format!("localhost:{}", config.port))?
        .run()
        .await
}

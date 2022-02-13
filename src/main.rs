use tokio::sync::mpsc;

use core::{event, mongo, rpc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut endpoint = rpc::EventDrivenClient::new(rx);
    endpoint.connect("").await;
    tokio::spawn(endpoint.start());

    let worker = table::core::svc::TableWorker::new(
        mongo::MongoDesc {
            uri: "mongodb://localhost:27017".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            database: "sheet_flow".to_string(),
        },
        tx,
    ).await;

    actix_web::HttpServer::new(||
        actix_web::App::new()
            .data(worker)
            .service(table::api::table_api))
        .bind(":8080")?
        .run()
        .await
}
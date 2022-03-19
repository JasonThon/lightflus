use tokio::sync::mpsc;

use dataflow::coord::Coordinator;
use dataflow::event;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::HttpServer::new(||
        actix_web::App::new()
            .data(Coordinator::new())
            .service(
                actix_web::web::scope("coordinator")
                    .service(dataflow::table_api)
                    .service(dataflow::overview)
            ))
        .bind("localhost:8080")?
        .run()
        .await
}
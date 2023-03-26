use std::time::Duration;

use actix_web::{web, App};
use handler::{
    resources::{create_resource, get_resource, list_resources, overview},
    RESOURCES_HANDLER_ROOT,
};
mod handler;
mod types;
mod err;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    actix_web::HttpServer::new(|| {
        App::new()
            .service(
                web::scope(RESOURCES_HANDLER_ROOT)
                    .service(create_resource)
                    .service(get_resource)
                    .service(list_resources),
            )
            .service(overview)
    })
    .client_disconnect_timeout(Duration::from_secs(3))
    .client_request_timeout(Duration::from_secs(3))
    .worker_max_blocking_threads(10)
    .workers(3)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

use actix_web::{web, App};
use handler::{
    resources::{create_resource, get_resource, list_resources, overview},
    RESOURCES_HANDLER_ROOT,
};
mod handler;
mod types;

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
    .bind(("localhost", 8080))?
    .run()
    .await
}

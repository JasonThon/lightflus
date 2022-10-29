use actix_web::{web, App};
use handler::{
    resources::{create_resource, get_resource, list_resources},
    resources_handler_root,
};
mod handler;
mod utils;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let resource_scope = web::scope(resources_handler_root)
        .service(create_resource)
        .service(get_resource)
        .service(list_resources);

    actix_web::HttpServer::new(|| App::new().service(resource_scope))
        .bind(("localhost", 8080))?
        .run()
        .await
}

use std::{env, time::Duration};

use actix_web::{web, App};

use lightflus_core::{
    apiserver::handler::{
        resources::{create_resource, get_resource, list_resources, overview},
        COORDINATOR_URI_ENV, RESOURCES_HANDLER_ROOT,
    },
    coordinator::{
        api::CoordinatorApiImpl,
        coord::{self, load_builder},
    },
};

use proto::coordinator::coordinator_api_server::CoordinatorApiServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let builder = &mut load_builder();

    replace_builder_args_by_env(builder);

    let coordinator = builder.build();

    let addr = format!("0.0.0.0:{}", builder.port).parse()?;
    env::set_var(COORDINATOR_URI_ENV, format!("localhost:{}", builder.port));

    let handler = tokio::spawn(
        actix_web::HttpServer::new(move || {
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
        .run(),
    );

    tracing::info!("service will start at {}", builder.port);

    Server::builder()
        .timeout(Duration::from_secs(3))
        .add_service(CoordinatorApiServer::new(CoordinatorApiImpl::new(
            coordinator,
        )))
        .serve(addr)
        .await?;

    handler.abort();

    Ok(())
}

fn replace_builder_args_by_env(_builder: &mut coord::CoordinatorBuilder) {}

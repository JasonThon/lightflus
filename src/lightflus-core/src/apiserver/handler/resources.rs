use std::sync::Arc;

use actix_web::{error::ErrorBadRequest, get, post, web, HttpResponse};
use common::utils::{from_pb_slice, pb_to_bytes_mut};
use futures_util::StreamExt;
use proto::apiserver::{CreateResourceRequest, CreateResourceResponse, ResourceTypeEnum};

use crate::apiserver::{
    handler::services::create_dataflow,
    types::{GetResourceArgs, ListResourcesArgs},
};

use super::services::get_dataflow;

#[post("/create")]
async fn create_resource(mut req: web::Payload) -> actix_web::Result<HttpResponse> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = req.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    match from_pb_slice::<CreateResourceRequest>(bytes.iter().as_slice()) {
        Ok(req) => match req.resource_type() {
            ResourceTypeEnum::Dataflow => create_dataflow(req)
                .await
                .map(|resp| HttpResponse::Created().body(pb_to_bytes_mut(resp))),
            _ => Ok(
                HttpResponse::Created().body(pb_to_bytes_mut(CreateResourceResponse::default()))
            ),
        },
        Err(err) => Err(ErrorBadRequest(err)),
    }
}

#[get("/get/{namespace}/{resource_type}/{resource_id}")]
async fn get_resource(args: web::Path<GetResourceArgs>) -> actix_web::Result<HttpResponse> {
    match ResourceTypeEnum::from_i32(args.resource_type) {
        Some(resource_type) => match resource_type {
            ResourceTypeEnum::Dataflow => get_dataflow(args.as_ref()).await,
            _ => Ok(HttpResponse::Ok().finish()),
        },
        None => Ok(HttpResponse::Ok().finish()),
    }
}

#[get("/list/{namespace}/{resource_type}")]
async fn list_resources(args: web::Path<ListResourcesArgs>) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().finish())
}

#[get("/overview")]
async fn overview() -> HttpResponse {
    HttpResponse::Ok().finish()
}

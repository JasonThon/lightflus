use actix_web::{error::ErrorBadRequest, get, post, web, HttpResponse};
use common::utils::{from_pb_slice, pb_to_bytes_mut};
use futures_util::StreamExt;
use proto::apiserver::{CreateResourceRequest, CreateResourceResponse, ResourceTypeEnum};

use crate::{
    handler::service::create_dataflow,
    types::{GetResourceArgs, ListResourcesArgs},
};

use super::{service::get_dataflow};

#[post("/create")]
async fn create_resource(mut req: web::Payload) -> actix_web::Result<HttpResponse> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = req.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    from_pb_slice::<CreateResourceRequest>(bytes.iter().as_slice())
        .map_err(|err| ErrorBadRequest(err))
        .and_then(|req| match req.resource_type() {
            ResourceTypeEnum::Dataflow => create_dataflow(req),
            _ => Ok(CreateResourceResponse::default()),
        })
        .map(|resp| HttpResponse::Created().body(pb_to_bytes_mut(resp)))
}

#[get("/get/{namespace}/{resource_type}/{resource_id}")]
async fn get_resource(args: web::Path<GetResourceArgs>) -> actix_web::Result<HttpResponse> {
    let resource_type_option = ResourceTypeEnum::from_i32(args.resource_type);

    if resource_type_option.is_some() {
        let ref resource_type = resource_type_option.unwrap();
        match resource_type {
            ResourceTypeEnum::Dataflow => get_dataflow(args.as_ref()).await,
            _ => Ok(HttpResponse::Ok().finish()),
        }
    } else {
        Ok(HttpResponse::Ok().finish())
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

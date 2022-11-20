use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, post, web, HttpResponse,
};
use common::{
    err::ApiError,
    utils::{from_pb_slice, pb_to_bytes_mut, uuid},
};
use futures_util::StreamExt;
use proto::{
    apiserver::{
        create_resource_request::Options, CreateResourceRequest, CreateResourceResponse,
        GetResourceResponse, Resource, ResourceStatusEnum, ResourceTypeEnum,
    },
    common::ResourceId,
    coordinator::{coordinator_api_client::CoordinatorApiClient, GetDataflowRequest},
};

use crate::types::{GetResourceArgs, ListResourcesArgs};

const COORDINATOR_URI_ENV: &str = "LIGHTFLUS_COORDINATOR_URI";

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
            ResourceTypeEnum::Dataflow => {
                let mut req = req.clone();
                let mut resource_id = ResourceId::default();
                resource_id.resource_id = uuid();
                req.options.iter_mut().for_each(|options| match options {
                    Options::Dataflow(dataflow) => dataflow
                        .dataflow
                        .iter_mut()
                        .for_each(|df| df.job_id = Some(resource_id)),
                });

                let uri = common::utils::get_env(COORDINATOR_URI_ENV);
                let mut cli = futures_executor::block_on(CoordinatorApiClient::connect(uri.unwrap_or_default()))?;

                let result = futures_executor::block_on(cli.create_dataflow(tonic::Request::new(req)));
                result
                    .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
                    .map(|_| {
                        let mut response = CreateResourceResponse::default();
                        response.set_status(ResourceStatusEnum::Starting);
                        response
                    })
            }
            _ => Ok(CreateResourceResponse::default()),
        })
        .map(|resp| HttpResponse::Created().body(pb_to_bytes_mut(resp)))
}

#[get("/get/{namespace}/{resource_type}/{resource_id}")]
async fn get_resource(args: web::Path<GetResourceArgs>) -> actix_web::Result<HttpResponse> {
    let resource_type_option = ResourceTypeEnum::from_i32(args.resource_type);
    let mut resp = HttpResponse::Ok();

    if resource_type_option.is_some() {
        let ref resource_type = resource_type_option.unwrap();
        match resource_type {
            ResourceTypeEnum::Dataflow => {
                let uri = common::utils::get_env(COORDINATOR_URI_ENV).unwrap();
                let cli = CoordinatorApiClient::connect(uri.unwrap_or_default()).await?;
                let ref mut req = GetDataflowRequest::default();
                req.job_id = Some(args.to_resource_id());
                cli.get_dataflow(req)
                    .await
                    .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
                    .and_then(|resp| {
                        let mut response = GetResourceResponse::default();
                        let mut resource = Resource::default();
                        match resp.into_inner().graph {
                            Some(dataflow) => {
                                resource.set_resource_id();
                                resource.set_resource_type(*resource_type);
                                response.set_resource(resource);
                                Ok(response)
                            }
                            None => ErrorBadRequest("empty graph response"),
                        }
                    })
                    .map(|response| resp.body(pb_to_bytes_mut(response)))
            }
            _ => Ok(resp.finish()),
        }
    } else {
        Ok(resp.finish())
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

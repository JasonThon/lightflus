use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    HttpResponse,
};
use common::{
    err::ApiError,
    utils::{pb_to_bytes_mut, uuid},
};
use proto::{
    apiserver::{
        create_resource_request::Options, CreateResourceRequest, CreateResourceResponse,
        GetResourceResponse, Resource, ResourceStatusEnum, ResourceTypeEnum,
    },
    common::ResourceId,
    coordinator::{coordinator_api_client::CoordinatorApiClient, GetDataflowRequest},
};

use crate::types::GetResourceArgs;

use super::COORDINATOR_URI_ENV;

pub(crate) fn create_dataflow(
    req: CreateResourceRequest,
) -> Result<CreateResourceResponse, actix_web::Error> {
    let mut req = req.clone();
    let mut resource_id = ResourceId::default();
    resource_id.resource_id = uuid();
    req.options.iter_mut().for_each(|options| match options {
        Options::Dataflow(dataflow) => dataflow
            .dataflow
            .iter_mut()
            .for_each(|df| df.job_id = Some(resource_id.clone())),
    });
    match req.options {
        Some(options) => match options {
            Options::Dataflow(dataflow) => match dataflow.dataflow {
                Some(dataflow) => {
                    let uri = common::utils::get_env(COORDINATOR_URI_ENV);
                    let ref mut cli = futures_executor::block_on(CoordinatorApiClient::connect(
                        uri.unwrap_or_default(),
                    ));

                    cli.as_mut()
                        .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
                        .and_then(|client| {
                            let result = futures_executor::block_on(
                                client.create_dataflow(tonic::Request::new(dataflow)),
                            );
                            result
                                .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
                                .map(|resp| {
                                    let mut response = CreateResourceResponse::default();
                                    response.set_status(ResourceStatusEnum::Starting);
                                    response
                                })
                        })
                }
                None => Err(ErrorBadRequest("empty dataflow")),
            },
        },
        None => Err(ErrorBadRequest("empty dataflow")),
    }
}

pub(crate) async fn get_dataflow(args: &GetResourceArgs) -> actix_web::Result<HttpResponse> {
    let mut resp = HttpResponse::Ok();
    let uri = common::utils::get_env(COORDINATOR_URI_ENV).unwrap();
    let mut cli = CoordinatorApiClient::connect(uri).await;

    cli.as_mut()
        .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
        .and_then(|client| {
            let mut req = GetDataflowRequest::default();
            req.job_id = Some(args.to_resource_id());
            futures_executor::block_on(client.get_dataflow(tonic::Request::new(req)))
                .map_err(|err| ErrorInternalServerError(ApiError::from(err)))
                .and_then(|resp| {
                    let mut response = GetResourceResponse::default();
                    let mut resource = Resource::default();
                    match resp.into_inner().graph {
                        Some(dataflow) => {
                            resource.resource_id = dataflow.job_id.clone();
                            resource.set_resource_type(ResourceTypeEnum::Dataflow);
                            response.resource = Some(resource);
                            Ok(response)
                        }
                        None => Err(ErrorBadRequest("empty graph response")),
                    }
                })
                .map(|response| resp.body(pb_to_bytes_mut(response)))
        })
}

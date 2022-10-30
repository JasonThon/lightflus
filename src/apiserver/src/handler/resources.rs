use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, post, web, HttpResponse,
};
use futures::StreamExt;
use proto::{
    apiserver::{
        CreateResourceRequest, CreateResourceResponse, GetResourceResponse, Resource,
        ResourceStatusEnum,
        ResourceTypeEnum::{self, RESOURCE_TYPE_ENUM_DATAFLOW},
    },
    coordinator::{cli::new_coordinator_client, coordinator::GetDataflowRequest},
};
use protobuf::{CodedInputStream, Message, ProtobufEnum, ProtobufError};

use crate::{
    types::{GetResourceArgs, ListResourcesArgs},
    utils::{grpc_err_to_actix_err, pb_to_bytes_mut},
};

const COORDINATOR_URI_ENV: &str = "LIGHTFLUS_COORDINATOR_URI";

#[post("/create")]
async fn create_resource(mut req: web::Payload) -> actix_web::Result<HttpResponse> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = req.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    let ref mut stream = CodedInputStream::from_bytes(bytes.iter().as_slice());
    CreateResourceRequest::parse_from(stream)
        .map_err(|err| match err {
            ProtobufError::IoError(io_err) => ErrorBadRequest(io_err),
            ProtobufError::WireError(wire_err) => ErrorBadRequest(format!("{}", wire_err)),
            ProtobufError::Utf8(utf8_err) => ErrorBadRequest(utf8_err),
            ProtobufError::MessageNotInitialized { message } => ErrorBadRequest(message),
        })
        .and_then(|req| match req.get_resource_type() {
            RESOURCE_TYPE_ENUM_DATAFLOW => {
                let uri = common::utils::get_env(COORDINATOR_URI_ENV);
                let cli = new_coordinator_client(uri.unwrap());
                let result = cli.create_dataflow_async(req.get_dataflow().get_dataflow());
                result
                    .map_err(|err| ErrorInternalServerError(err))
                    .map(|_| {
                        let mut response = CreateResourceResponse::default();
                        response.set_status(ResourceStatusEnum::RESOURCE_STATUS_ENUM_STARTING);
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
            RESOURCE_TYPE_ENUM_DATAFLOW => {
                let uri = common::utils::get_env(COORDINATOR_URI_ENV).unwrap();
                let cli = new_coordinator_client(uri);
                let ref mut req = GetDataflowRequest::default();
                req.set_job_id(args.to_resource_id());
                cli.get_dataflow(req)
                    .map_err(|err| grpc_err_to_actix_err(err))
                    .and_then(|resp| {
                        let mut response = GetResourceResponse::default();
                        let mut resource = Resource::default();
                        resource.set_resource_id(resp.get_graph().get_job_id().clone());
                        resource.set_resource_type(*resource_type);
                        response.set_resource(resource);
                        Ok(response)
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

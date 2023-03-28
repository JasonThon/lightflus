use common::net::gateway::taskmanager::SafeTaskManagerRpcGateway;
use lightflus_core::taskmanager::rpc::TaskManagerBuilder;
use proto::{
    common::{Dataflow, HostAddr, ResourceId},
    taskmanager::CreateSubDataflowRequest,
};
use tokio::task::JoinHandle;
use tonic::transport::{Error, Server};

fn setup_builder() -> TaskManagerBuilder {
    TaskManagerBuilder {
        port: 8793,
        max_job_nums: 10,
    }
}

fn setup_server() -> JoinHandle<Result<(), Error>> {
    let builder = setup_builder();
    let server = builder.build();
    let addr = format!("0.0.0.0:{}", builder.port).parse().expect("msg");
    tokio::spawn(Server::builder().add_service(server).serve(addr))
}

#[tokio::test]
async fn test_taskmanager_deploy_and_terminate() {
    let handler = setup_server();

    let gateway = SafeTaskManagerRpcGateway::new(&HostAddr {
        host: "localhost".to_string(),
        port: 8793,
    });

    let mut dataflow = Dataflow::default();

    let r = gateway
        .create_sub_dataflow(CreateSubDataflowRequest {
            job_id: Some(ResourceId {
                resource_id: "rs_id".to_string(),
                namespace_id: "ns_id".to_string(),
            }),
            dataflow: Some(dataflow),
        })
        .await;
    assert!(r.is_ok());

    handler.abort();
}

#[tokio::test]
async fn test_taskmanager_send_event_to_operator() {
    let handler = setup_server();

    let gateway = SafeTaskManagerRpcGateway::new(&HostAddr {
        host: "localhost".to_string(),
        port: 8793,
    });
}

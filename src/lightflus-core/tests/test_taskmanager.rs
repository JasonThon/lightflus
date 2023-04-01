use std::{collections::HashMap, sync::Once};

use common::net::gateway::taskmanager::SafeTaskManagerRpcGateway;
use lightflus_core::taskmanager::rpc::TaskManagerBuilder;
use proto::{
    common::{
        mapper, operator_info, Dataflow, DataflowMeta, ExecutorStatus, Func, HostAddr, Mapper,
        OperatorInfo, ResourceId,
    },
    taskmanager::CreateSubDataflowRequest,
};
use stream::initialize_v8;
use tokio::task::JoinHandle;
use tonic::transport::{Error, Server};

fn setup_builder(port: usize) -> TaskManagerBuilder {
    TaskManagerBuilder {
        port,
        max_job_nums: 10,
    }
}

static ONCE: Once = Once::new();

pub fn setup() {
    ONCE.call_once(|| {
        tracing_subscriber::fmt::init();
        initialize_v8();
    })
}


fn setup_server(port: usize) -> JoinHandle<Result<(), Error>> {
    let builder = setup_builder(port);
    let server = builder.build();
    let addr = format!("0.0.0.0:{}", builder.port).parse().expect("msg");
    tokio::spawn(Server::builder().add_service(server).serve(addr))
}

fn setup_dataflow(resource_id: ResourceId, server_port: usize) -> Dataflow {
    let mut dataflow = Dataflow::default();
    dataflow.job_id = Some(resource_id);

    dataflow.meta = vec![DataflowMeta {
        center: 0,
        neighbors: vec![1],
    }];

    dataflow.nodes = HashMap::from_iter([
        (
            0,
            OperatorInfo {
                operator_id: 0,
                host_addr: Some(HostAddr {
                    host: "localhost".to_string(),
                    port: server_port as u32,
                }),
                upstreams: vec![],
                details: Some(operator_info::Details::Mapper(Mapper {
                    value: Some(mapper::Value::Func(Func {
                        function: format!("_operator_{}_process", "map"),
                    })),
                })),
            },
        ),
        (
            1,
            OperatorInfo {
                operator_id: 1,
                host_addr: Some(HostAddr {
                    host: "localhost".to_string(),
                    port: server_port as u32,
                }),
                upstreams: vec![0],
                details: Some(operator_info::Details::Mapper(Mapper {
                    value: Some(mapper::Value::Func(Func {
                        function: format!("_operator_{}_process", "map"),
                    })),
                })),
            },
        ),
    ]);
    dataflow
}

#[tokio::test]
async fn test_taskmanager_deploy_and_terminate() {
    setup();
    let server_port = 8793;
    let server_1 = setup_server(server_port);

    let gateway = SafeTaskManagerRpcGateway::new(&HostAddr {
        host: "localhost".to_string(),
        port: 8793,
    });

    let dataflow = setup_dataflow(
        ResourceId {
            resource_id: "rs_id".to_string(),
            namespace_id: "ns_id".to_string(),
        },
        server_port,
    );

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

    let r = gateway
        .get_sub_dataflow(ResourceId {
            resource_id: "rs_id".to_string(),
            namespace_id: "ns_id".to_string(),
        })
        .await;
    assert!(r.is_ok());

    let states = r.expect("msg");
    assert!(states.subdataflow_infos.is_some());
    let infos = states.subdataflow_infos.unwrap_or_default();
    assert!(infos.execution_id.is_some());
    assert!(!infos.executors_info.is_empty());

    infos
        .executors_info
        .iter()
        .for_each(|(_, info)| assert_eq!(info.status(), ExecutorStatus::Running));

    let r = gateway
        .stop_dataflow(ResourceId {
            resource_id: "rs_id".to_string(),
            namespace_id: "ns_id".to_string(),
        })
        .await;
    assert!(r.is_ok());

    server_1.abort();
}
use std::{collections::HashMap, sync::Once, time::Duration};

use common::{
    kafka::run_producer,
    net::{
        cluster::ClusterBuilder, gateway::coordinator::SafeCoordinatorRpcGateway,
        AckResponderBuilder, HeartbeatBuilder,
    },
    redis::RedisClient,
    types::TypedValue,
};
use lightflus_core::{
    coordinator::{
        api::CoordinatorApiImpl, coord::CoordinatorBuilder, storage::DataflowStorageBuilder,
    },
    taskmanager::rpc::TaskManagerBuilder,
};
use proto::{
    common::{
        flat_map, kafka_desc, key_by, operator_info, redis_desc::ConnectionOpts, reducer, sink,
        source, DataTypeEnum, Dataflow, DataflowMeta, DeliveryGuarentee, FlatMap, Func, HostAddr,
        KafkaDesc, KeyBy, OperatorInfo, RedisDesc, Reducer, ResourceId, Sink, Source,
    },
    coordinator::coordinator_api_server::CoordinatorApiServer,
};
use stream::initialize_v8;
use tokio::{task::JoinHandle, time};
use tonic::transport::{Error, Server};

static ONCE: Once = Once::new();

pub fn setup() {
    ONCE.call_once(|| {
        tracing_subscriber::fmt::init();
        initialize_v8();
    })
}

fn setup_wordcount_dataflow(worker_port: u32) -> Dataflow {
    Dataflow {
        job_id: Some(ResourceId {
            resource_id: "rsId".to_string(),
            namespace_id: "nsId".to_string(),
        }),
        meta: vec![
            DataflowMeta {
                center: 0,
                neighbors: vec![1],
            },
            DataflowMeta {
                center: 1,
                neighbors: vec![2],
            },
            DataflowMeta {
                center: 2,
                neighbors: vec![3],
            },
            DataflowMeta {
                center: 3,
                neighbors: vec![4],
            },
            DataflowMeta {
                center: 4,
                neighbors: vec![],
            },
        ],
        nodes: HashMap::from_iter([
            (
                0,
                OperatorInfo {
                    operator_id: 0,
                    host_addr: Some(HostAddr {
                        host: "localhost".to_string(),
                        port: worker_port,
                    }),
                    upstreams: vec![],
                    details: Some(operator_info::Details::Source(Source {
                        desc: Some(source::Desc::Kafka(KafkaDesc {
                            brokers: vec!["localhost:9092".to_string()],
                            topic: "topic-1".to_string(),
                            opts: Some(kafka_desc::KafkaOptions {
                                group: Some("word_count".to_string()),
                                partition: None,
                            }),
                            data_type: DataTypeEnum::String as i32,
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
                        port: worker_port,
                    }),
                    upstreams: vec![0],
                    details: Some(operator_info::Details::FlatMap(FlatMap {
                        value: Some(flat_map::Value::Func(Func {
                            function: [
                                format!("function _operator_{}_process(v) ", "flatMap"),
                                "{ return v.split(\" \").map(v => {
                            return { t0: 1, t1: v };
                          })}"
                                .to_string(),
                            ]
                            .concat(),
                        })),
                    })),
                },
            ),
            (
                2,
                OperatorInfo {
                    operator_id: 2,
                    host_addr: Some(HostAddr {
                        host: "localhost".to_string(),
                        port: worker_port,
                    }),
                    upstreams: vec![1],
                    details: Some(operator_info::Details::KeyBy(KeyBy {
                        value: Some(key_by::Value::Func(Func {
                            function: [
                                format!("function _operator_{}_process(v) ", "keyBy"),
                                "{ return v.t1 }".to_string(),
                            ]
                            .concat(),
                        })),
                    })),
                },
            ),
            (
                3,
                OperatorInfo {
                    operator_id: 3,
                    host_addr: Some(HostAddr {
                        host: "localhost".to_string(),
                        port: worker_port,
                    }),
                    upstreams: vec![2],
                    details: Some(operator_info::Details::Reducer(Reducer {
                        value: Some(reducer::Value::Func(Func {
                            function: [
                                format!("function _operator_{}_process(v1, v2) ", "reduce"),
                                "{ return { t1: v1.t1, t0: v1.t0 + v2.t0 }; }".to_string(),
                            ]
                            .concat(),
                        })),
                    })),
                },
            ),
            (
                4,
                OperatorInfo {
                    operator_id: 4,
                    host_addr: Some(HostAddr {
                        host: "localhost".to_string(),
                        port: worker_port,
                    }),
                    upstreams: vec![3],
                    details: Some(operator_info::Details::Sink(Sink {
                        delivery_guarentee: DeliveryGuarentee::None as i32,
                        desc: Some(sink::Desc::Redis(RedisDesc {
                            connection_opts: Some(ConnectionOpts {
                                host: "localhost:6379".to_string(),
                                username: "".to_string(),
                                password: "".to_string(),
                                database: 0,
                                tls: false,
                            }),
                            key_extractor: Some(Func {
                                function: "function redis_extractor(a) { return a.t1 }".to_string(),
                            }),
                            value_extractor: Some(Func {
                                function: "function redis_extractor(a) { return a.t0.toString() }"
                                    .to_string(),
                            }),
                        })),
                    })),
                },
            ),
        ]),
        execution_id: None,
    }
}

fn setup_builder(port: usize) -> TaskManagerBuilder {
    TaskManagerBuilder {
        port,
        max_job_nums: 10,
    }
}

fn setup_server(port: usize) -> JoinHandle<Result<(), Error>> {
    let builder = setup_builder(port);
    let server = builder.build();
    let addr = format!("0.0.0.0:{}", builder.port).parse().expect("msg");
    tokio::spawn(Server::builder().add_service(server).serve(addr))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 20)]
async fn test_e2e() {
    setup();
    let taskmanager_port = 9999;
    let builder = CoordinatorBuilder {
        port: 8888,
        cluster: ClusterBuilder {
            nodes: format!("localhost:{}", taskmanager_port),
            rpc_timeout: 5,
            connect_timeout: 5,
        },
        storage: DataflowStorageBuilder::Memory,
        heartbeat: HeartbeatBuilder {
            period: 3,
            connect_timeout: 3,
            rpc_timeout: 3,
        },
        ack: AckResponderBuilder {
            delay: 3,
            buf_size: 10,
            connect_timeout: 5,
            rpc_timeout: 5,
        },
    };

    let addr = format!("0.0.0.0:{}", builder.port).parse().expect("msg");
    let coordinator = builder.build();
    let _ = tokio::spawn(
        Server::builder()
            .timeout(Duration::from_secs(3))
            .add_service(CoordinatorApiServer::new(CoordinatorApiImpl::new(
                coordinator,
            )))
            .serve(addr),
    );

    let _ = setup_server(taskmanager_port as usize);

    let gateway = SafeCoordinatorRpcGateway::new(&HostAddr {
        host: "localhost".to_string(),
        port: builder.port as u32,
    })
    .await;
    let producer = run_producer("localhost:9092", "topic-1", "", 0).expect("msg");

    let r = time::timeout(
        Duration::from_secs(3),
        producer.send(
            "key".as_bytes(),
            "word word word count count count".as_bytes(),
        ),
    )
    .await;
    assert!(r.is_ok());
    println!("success send event");

    let dataflow = setup_wordcount_dataflow(taskmanager_port as u32);

    let r = gateway.create_dataflow(dataflow).await;
    assert!(r.is_ok());
    println!("success create dataflow");

    let desc = RedisDesc {
        connection_opts: Some(ConnectionOpts {
            host: "localhost:6379".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            database: 0,
            tls: false,
        }),
        key_extractor: Some(Func {
            function: "function redis_extractor(a) { return a.t1 }".to_string(),
        }),
        value_extractor: Some(Func {
            function: "function redis_extractor(a) { return a.t0.toString() }".to_string(),
        }),
    };
    let mut redis = RedisClient::new(&desc);
    let _ = tokio::time::sleep(Duration::from_secs(3)).await;

    let r = redis.get(&TypedValue::String("word".to_string()));

    assert!(r.is_ok());
    let r = r.expect("msg");
    let v = TypedValue::from_slice_with_type(&r, DataTypeEnum::String);
    assert_eq!(v, TypedValue::String("3".to_string()));

    let r = redis.get(&TypedValue::String("count".to_string()));

    assert!(r.is_ok());
    let r = r.expect("msg");
    let v = TypedValue::from_slice_with_type(&r, DataTypeEnum::String);
    assert_eq!(v, TypedValue::String("3".to_string()));
    println!("success sink");
    return;
}

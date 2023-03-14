use std::{
    collections::{btree_set::Iter, BTreeMap, BTreeSet, VecDeque},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use common::{
    consts::{
        default_configs::{
            DEFAULT_CHANNEL_SIZE, DEFAULT_SEND_OPERATOR_EVENT_CONNECT_TIMEOUT_MILLIS,
            DEFAULT_SEND_OPERATOR_EVENT_RPC_TIMEOUT_MILLIS,
        },
        env_keys::{
            CHANNEL_SIZE, SEND_OPERATOR_EVENT_CONNECT_TIMEOUT, SEND_OPERATOR_EVENT_RPC_TIMEOUT,
        },
    },
    event::LocalEvent,
    futures::{join_all, select},
    map_iter,
    net::gateway::taskmanager::SafeTaskManagerRpcGateway,
    types::{ExecutorId, SinkId},
    utils::get_env,
};
use futures_util::FutureExt;
use futures_util::{future::Either, ready};
use proto::common::{
    operator_info::Details, Ack, DataflowMeta, KeyedDataEvent, OperatorInfo, ResourceId,
};
use tokio::task::JoinHandle;

use crate::{
    connector::{Sink, SinkImpl, Source, SourceImpl},
    dataflow::Execution,
    edge::{InEdge, LocalInEdge, LocalOutEdge, OutEdge, RemoteOutEdge},
    err::ExecutionError,
    new_event_channel,
    state::{new_state_mgt, StateManager},
    window::WindowAssignerImpl,
    Receiver, Sender,
};

pub struct Task {
    executor_id: ExecutorId,
    job_id: ResourceId,
    main_executor_handle: Option<JoinHandle<()>>,
    downstream: BTreeSet<ExecutorId>,
}

impl Task {
    pub fn receive_ack(&self, ack: &Ack) {}

    pub fn new(job_id: &ResourceId, adjacent_node: &DataflowMeta) -> Self {
        Self {
            executor_id: adjacent_node.center,
            job_id: job_id.clone(),
            main_executor_handle: None,
            downstream: adjacent_node.neighbors.iter().map(|id| *id).collect(),
        }
    }

    pub fn get_downstream_id_iter(&self) -> Iter<ExecutorId> {
        self.downstream.iter()
    }

    pub fn create_stream_executor(&self, operator_info: &OperatorInfo) -> StreamExecutor {
        let details = operator_info.details.clone().unwrap();
        let source = if operator_info.has_source() {
            Some(SourceImpl::from((
                &self.job_id,
                operator_info.operator_id,
                &details,
            )))
        } else {
            None
        };

        StreamExecutor {
            external_sinks: Default::default(),
            executor_id: self.executor_id,
            out_edges: Default::default(),
            in_edge: None,
            source,
            operator_details: details,
            job_id: self.job_id.clone(),
        }
    }

    pub fn start(&mut self, executor: StreamExecutor) {
        self.main_executor_handle = Some(tokio::spawn(executor))
    }
}

pub enum EdgeBuilder<'a> {
    Local {
        tx: Sender<bytes::Bytes>,
        rx: Receiver<bytes::Bytes>,
        operator_info: &'a OperatorInfo,
    },
    Remote {
        gateway: SafeTaskManagerRpcGateway,
    },
}

impl<'a> EdgeBuilder<'a> {
    pub fn local(operator_info: &'a OperatorInfo) -> Self {
        let channel_size = get_env(CHANNEL_SIZE)
            .and_then(|size| size.parse::<usize>().ok())
            .unwrap_or(DEFAULT_CHANNEL_SIZE);
        let (tx, rx) = new_event_channel(channel_size);
        Self::Local {
            tx,
            rx,
            operator_info,
        }
    }

    pub fn remote(operator_info: &'a OperatorInfo) -> Self {
        let host_addr = operator_info.get_host_addr_ref().unwrap();
        let connect_timeout = get_env(SEND_OPERATOR_EVENT_CONNECT_TIMEOUT)
            .and_then(|size| size.parse::<u64>().ok())
            .unwrap_or(DEFAULT_SEND_OPERATOR_EVENT_CONNECT_TIMEOUT_MILLIS);
        let rpc_timeout = get_env(SEND_OPERATOR_EVENT_RPC_TIMEOUT)
            .and_then(|size| size.parse::<u64>().ok())
            .unwrap_or(DEFAULT_SEND_OPERATOR_EVENT_RPC_TIMEOUT_MILLIS);
        Self::Remote {
            gateway: SafeTaskManagerRpcGateway::with_timeout(
                host_addr,
                Duration::from_secs(connect_timeout),
                Duration::from_secs(rpc_timeout),
            ),
        }
    }

    pub fn build_out_edge(&self) -> Box<dyn OutEdge<Output = LocalEvent>> {
        match self {
            Self::Local { tx, .. } => Box::new(LocalOutEdge::<LocalEvent>::new(tx.clone())),
            Self::Remote { gateway } => Box::new(RemoteOutEdge::new(gateway.clone())),
        }
    }

    /// Unlike out-edge which the data stream can be broadcast to multiple downstreams, each operator does have only on in-edge to receive data stream.
    /// For different edge type, [EdgeBuilder] will return two different values:
    ///
    /// - [EdgeBuilder::Local] can create in-edge and return [Some]
    /// - then else will return [None]
    pub fn build_in_edge(self) -> Option<Pin<Box<dyn InEdge<Output = LocalEvent>>>> {
        match self {
            Self::Local { tx: _, rx, .. } => Some(Box::pin(LocalInEdge::<LocalEvent>::new(rx))),
            _ => None,
        }
    }
}

/// The executor
pub struct StreamExecutor {
    external_sinks: BTreeMap<SinkId, SinkImpl>,
    executor_id: ExecutorId,
    out_edges: BTreeMap<ExecutorId, Box<dyn OutEdge<Output = LocalEvent>>>,
    in_edge: Option<Pin<Box<dyn InEdge<Output = LocalEvent>>>>,
    source: Option<SourceImpl>,
    operator_details: Details,
    job_id: ResourceId,
}

impl StreamExecutor {
    pub fn add_external_sink(&mut self, sink: SinkImpl) {
        self.external_sinks.insert(sink.sink_id(), sink);
    }

    pub fn add_out_edge(
        &mut self,
        executor_id: ExecutorId,
        out_edge: Box<dyn OutEdge<Output = LocalEvent>>,
    ) {
        self.out_edges.insert(executor_id, out_edge);
    }

    pub fn set_in_edge(&mut self, in_edge: Option<Pin<Box<dyn InEdge<Output = LocalEvent>>>>) {
        self.in_edge = in_edge;
    }

    pub fn poll_recv_data_stream(
        &mut self,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Option<LocalEvent>> {
        if self.in_edge.is_some() {
            match &mut self.in_edge {
                Some(in_edge) => in_edge.poll_recv_data_stream(cx),
                None => std::task::Poll::Pending,
            }
        } else if self.source.is_some() {
            match &mut self.source {
                Some(source) => match source.poll_recv_msg(cx) {
                    std::task::Poll::Ready(message) => std::task::Poll::Ready(message),
                    std::task::Poll::Pending => std::task::Poll::Pending,
                },
                None => std::task::Poll::Ready(None),
            }
        } else {
            std::task::Poll::Ready(None)
        }
    }

    #[inline]
    fn process<T: StateManager>(
        &mut self,
        execution: &Execution<'_, '_, T>,
        event: KeyedDataEvent,
        cx: &mut Context<'_>,
    ) {
        match execution.process(&event) {
            Ok(events) => self.sink_to_external_and_local(events, cx),
            Err(err) => match err {
                ExecutionError::OperatorUnimplemented(_) => {
                    self.sink_to_external_and_local(vec![event], cx)
                },
                _ => tracing::error!("process event failed: job_id: {:?}, operator_id: {}, event: {:?}. error details: {}", &self.job_id,self.executor_id, event, err)
            },
        }
    }

    #[inline]
    fn sink_to_external_and_local(&self, events: Vec<KeyedDataEvent>, cx: &mut Context<'_>) {
        // clone events here is not exhaustable because Entry is a zero-copy structure.
        let ref mut sink_futures = map_iter!(self.external_sinks, |(executor_id, sink)| {
            sink.batch_sink(
                events
                    .iter()
                    .map(|event| LocalEvent::KeyedDataStreamEvent(event.clone()))
                    .map(|mut event| {
                        event.set_to_operator_id(*executor_id);
                        event
                    })
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

        let ref mut out_edge_futures = map_iter!(self.out_edges, |(executor_id, out_edge)| {
            out_edge.batch_write(
                events
                    .iter()
                    .map(|event| LocalEvent::KeyedDataStreamEvent(event.clone()))
                    .map(|mut event| {
                        event.set_to_operator_id(*executor_id);
                        event
                    })
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

        join_all(cx, sink_futures, |r| match r {
            Ok(_) => {}
            Err(err) => tracing::error!("{:?}", err),
        });

        join_all(cx, out_edge_futures, |r| match r {
            Ok(_) => {}
            Err(err) => tracing::error!("{:?}", err),
        })
    }
}

/// implement the executor runner
impl Future for StreamExecutor {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let ref mut this_windows = VecDeque::default();
        match &this.operator_details {
            Details::Window(window) => {
                let mut assigner = WindowAssignerImpl::new(window);
                loop {
                    match select(this.poll_recv_data_stream(cx), assigner.poll_trigger(cx)) {
                        Some(either) => match either {
                            Either::Left(opt) => match opt {
                                Some(event) => match event {
                                    LocalEvent::Terminate { .. } => return Poll::Ready(()),
                                    LocalEvent::KeyedDataStreamEvent(event) => {
                                        let windows = assigner.assign_windows(event);
                                        this_windows.extend(windows)
                                    }
                                },
                                None => {}
                            },
                            Either::Right(_) => this.sink_to_external_and_local(
                                assigner
                                    .group_by_key_and_window(this_windows)
                                    .into_iter()
                                    .map(|window| window.to_event())
                                    .collect(),
                                cx,
                            ),
                        },
                        None => {}
                    }
                }
            }
            _ => {
                let execution = Execution::new(
                    this.executor_id,
                    &this.operator_details,
                    new_state_mgt(&this.job_id),
                    scope,
                );

                loop {
                    while let Some(event) = ready!(this.poll_recv_data_stream(cx)) {
                        match event {
                            LocalEvent::KeyedDataStreamEvent(keyed_event) => {
                                this.process(&execution, keyed_event, cx)
                            }
                            _ => return std::task::Poll::Ready(()),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{event::LocalEvent, types::TypedValue, utils::times::now_timestamp};
    use proto::common::{
        mapper, operator_info, source, DataTypeEnum, DataflowMeta, Entry, Func, KafkaDesc,
        KeyedDataEvent, Mapper, OperatorInfo, ResourceId, Source,
    };

    use crate::{
        edge::{InEdge, LocalInEdge, LocalOutEdge, OutEdge},
        new_event_channel,
    };

    use super::{StreamExecutor, Task};

    struct TestStreamExecutorSuite {
        pub executor: StreamExecutor,
        pub in_edge_tx_endpoint: LocalOutEdge<LocalEvent>,
        pub out_edge_rx_endpoint: LocalInEdge<LocalEvent>,
    }

    struct SetupGuard {}

    impl Drop for SetupGuard {
        fn drop(&mut self) {}
    }

    fn setup() -> SetupGuard {
        use crate::MOD_TEST_START;
        MOD_TEST_START.call_once(|| {
            v8::V8::set_flags_from_string(
                "--no_freeze_flags_after_init --expose_gc --harmony-import-assertions --harmony-shadow-realm --allow_natives_syntax --turbo_fast_api_calls",
              );
                  v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
                  v8::V8::initialize();
        });
        std::env::set_var("STATE_MANAGER", "MEM");

        SetupGuard {}
    }

    #[test]
    fn test_task_get_downstream_id_iter() {
        let job_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespace_id".to_string(),
        };

        let meta = DataflowMeta {
            center: 0,
            neighbors: vec![1, 2, 3, 4],
        };
        let task = Task::new(&job_id, &meta);

        let mut index = 1;
        task.get_downstream_id_iter().for_each(|x| {
            assert_eq!(*x, index);
            index += 1;
        });
    }

    #[tokio::test]
    async fn test_task_create_stream_executor() {
        let job_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespace_id".to_string(),
        };

        let meta = DataflowMeta {
            center: 0,
            neighbors: vec![1, 2, 3, 4],
        };
        let task = Task::new(&job_id, &meta);
        let executor = task.create_stream_executor(&OperatorInfo {
            operator_id: 0,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(operator_info::Details::Source(Source {
                desc: Some(source::Desc::Kafka(KafkaDesc::default())),
            })),
        });

        assert_eq!(&executor.job_id, &job_id);
        assert!(executor.source.is_some());
        assert!(executor.in_edge.is_none());
        assert!(executor.external_sinks.is_empty());
        assert!(executor.out_edges.is_empty());
    }

    #[tokio::test]
    async fn test_stream_executor_process() {
        let _ = setup();
        let job_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespace_id".to_string(),
        };

        let meta = DataflowMeta {
            center: 0,
            neighbors: vec![1, 2, 3, 4],
        };
        let task = Task::new(&job_id, &meta);
        let mut executor = task.create_stream_executor(&OperatorInfo {
            operator_id: 1,
            host_addr: None,
            upstreams: Default::default(),
            details: Some(operator_info::Details::Mapper(Mapper {
                value: Some(mapper::Value::Func(Func {
                    function: "function _operator_map_process(a) { return a+1 }".to_string(),
                })),
            })),
        });

        let (tx, rx) = new_event_channel(10);
        {
            executor.set_in_edge(Some(Box::pin(LocalInEdge::new(rx))));
            assert!(executor.in_edge.is_some());
        }

        let in_edge_tx_endpoint = LocalOutEdge::new(tx);
        let (tx, rx) = new_event_channel(10);

        {
            executor.add_out_edge(2, Box::new(LocalOutEdge::new(tx)));
            assert_eq!(executor.out_edges.len(), 1);
        }

        let out_edge_rx_endpoint = LocalInEdge::new(rx);

        let mut suite = TestStreamExecutorSuite {
            executor,
            in_edge_tx_endpoint,
            out_edge_rx_endpoint,
        };

        let handler = tokio::spawn(suite.executor);
        let timestamp = now_timestamp();

        {
            let result = suite
                .in_edge_tx_endpoint
                .write(LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
                    job_id: Some(ResourceId {
                        resource_id: "resource_id".to_string(),
                        namespace_id: "ns_id".to_string(),
                    }),
                    key: None,
                    to_operator_id: 2,
                    data: vec![Entry {
                        data_type: DataTypeEnum::Number as i32,
                        value: TypedValue::Number(1.0).get_data_bytes(),
                    }],
                    event_time: timestamp,
                    from_operator_id: 0,
                    window: None,
                    event_id: 0,
                }))
                .await;
            assert!(result.is_ok());
        }

        {
            let opt = suite.out_edge_rx_endpoint.receive_data_stream().await;
            assert_eq!(
                opt,
                Some(LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
                    job_id: Some(ResourceId {
                        resource_id: "resource_id".to_string(),
                        namespace_id: "ns_id".to_string(),
                    }),
                    key: None,
                    to_operator_id: 2,
                    data: vec![Entry {
                        data_type: DataTypeEnum::Number as i32,
                        value: TypedValue::Number(2.0).get_data_bytes(),
                    }],
                    event_time: timestamp,
                    from_operator_id: 0,
                    window: None,
                    event_id: 0,
                }))
            );
        }

        handler.abort()
    }

    #[tokio::test]
    async fn test_stream_executor_window() {}
}

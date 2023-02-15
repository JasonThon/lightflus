use std::{
    collections::{btree_set::Iter, BTreeMap, BTreeSet},
    future::Future,
    pin::Pin,
    task::Context,
    time::Duration,
};

use common::{
    collections::lang,
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
    net::gateway::taskmanager::UnsafeTaskManagerRpcGateway,
    types::{ExecutorId, SinkId},
    utils::get_env,
};
use futures_util::ready;
use futures_util::FutureExt;
use proto::common::{
    operator_info::Details, DataflowMeta, KeyedDataEvent, OperatorInfo, ResourceId,
};
use tokio::task::JoinHandle;

use crate::{
    connector::{Sink, SinkImpl, Source, SourceImpl},
    dataflow::Execution,
    edge::{InEdge, LocalInEdge, LocalOutEdge, OutEdge, RemoteOutEdge},
    new_event_channel,
    state::{new_state_mgt, StateManager},
    Receiver, Sender,
};

pub struct Task {
    executor_id: ExecutorId,
    job_id: ResourceId,
    main_executor_handle: Option<JoinHandle<()>>,
    downstream: BTreeSet<ExecutorId>,
}

impl Task {
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
            Some(SourceImpl::from(&details))
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
        gateway: UnsafeTaskManagerRpcGateway,
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
            gateway: UnsafeTaskManagerRpcGateway::with_timeout(
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
    /// - the else will return [None]
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

    fn process<T: StateManager>(
        &mut self,
        execution: &Execution<'_, '_, T>,
        event: KeyedDataEvent,
        cx: &mut Context<'_>,
    ) {
        match execution.process(&event) {
            Ok(events) => {
                // clone events here is not exhaustable because Entry is a zero-copy structure.
                let mut futures = events
                    .into_iter()
                    .map(|event| LocalEvent::KeyedDataStreamEvent(event))
                    .map(|mut event| {
                        let sink_futures = self
                            .external_sinks
                            .iter()
                            .map(|(executor_id, sink)| {
                                match &mut event {
                                    LocalEvent::KeyedDataStreamEvent(e) => {
                                        e.to_operator_id = *executor_id;
                                    }
                                    _ => {}
                                };

                                sink.sink(event.clone())
                            })
                            .collect::<Vec<_>>();
                        let out_edge_futures = self
                            .out_edges
                            .iter()
                            .map(|(executor_id, edge)| {
                                match &mut event {
                                    LocalEvent::KeyedDataStreamEvent(e) => {
                                        e.to_operator_id = *executor_id
                                    }
                                    _ => {}
                                };

                                edge.send(event.clone())
                            })
                            .collect::<Vec<_>>();
                        (sink_futures, out_edge_futures)
                    })
                    .collect::<Vec<_>>();

                while let true = lang::any_match_mut(&mut futures, |(sink_fut, out_edge_fut)| {
                    lang::any_match_mut(sink_fut, |fut| {
                        let result = fut.poll_unpin(cx);
                        match result {
                            std::task::Poll::Ready(r) => {
                                match r {
                                    Err(err) => {
                                        tracing::error!(
                                            "event job {:?} sink to external sink failed. details: {:?}",
                                            &self.job_id,
                                            err
                                        );
                                    }
                                    _ => {}
                                }
                                false
                            }
                            std::task::Poll::Pending => true,
                        }
                    }) && lang::any_match_mut(out_edge_fut, |fut| {
                        let result = fut.poll_unpin(cx);
                        match result {
                            std::task::Poll::Ready(r) => {
                                match r {
                                    Err(err) => {
                                        tracing::error!(
                                            "event job {:?} sink to external sink failed. details: {:?}",
                                            &self.job_id,
                                            err
                                        );
                                    }
                                    _ => {}
                                }
                                false
                            }
                            std::task::Poll::Pending => true,
                        }
                    })
                }) {}
            }
            Err(err) => tracing::error!("{}", err),
        }
    }
}

/// implement the executor runner
impl Future for StreamExecutor {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let execution = Execution::new(
            this.executor_id,
            &this.operator_details,
            new_state_mgt(&this.job_id),
            scope,
        );

        loop {
            while let Some(event) = ready!(this.poll_recv_data_stream(cx)) {
                match event {
                    LocalEvent::Terminate { .. } => return std::task::Poll::Ready(()),
                    LocalEvent::KeyedDataStreamEvent(keyed_event) => {
                        this.process(&execution, keyed_event, cx)
                    }
                }
            }
        }
    }
}

use crate::dataflow::Execution;
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::{new_state_mgt, StateManager};
use crate::v8_runtime::RuntimeEngine;
use crate::window::{KeyedWindow, WindowAssignerImpl};
use crate::{new_event_channel, EventReceiver, EventSender, DEFAULT_CHANNEL_SIZE};

use async_trait::async_trait;
use common::collections::lang;
use common::db::MysqlConn;
use common::event::{LocalEvent, SinkableMessage, SinkableMessageImpl};
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer};

use common::net::gateway::taskmanager::SafeTaskManagerRpcGateway;
use common::redis::RedisClient;
use common::types::{ExecutorId, SinkId, SourceId, TypedValue};
use common::utils::{self, get_env};
use prost::Message;
use prost_types::Timestamp;

use proto::common::ResourceId;
use proto::common::{sink, source, DataflowMeta, KafkaDesc, MysqlDesc, OperatorInfo, RedisDesc};
use proto::common::{Entry, KeyedDataEvent};

use proto::taskmanager::SendEventToOperatorStatusEnum;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::Mutex;

use std::collections::{BTreeMap, VecDeque};

use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::vec;

#[derive(Clone, Debug, PartialEq)]
pub struct DataflowContext {
    pub job_id: ResourceId,
    pub meta: Vec<DataflowMeta>,
    pub nodes: BTreeMap<ExecutorId, OperatorInfo>,
}

impl DataflowContext {
    pub fn new(
        job_id: ResourceId,
        meta: Vec<DataflowMeta>,
        nodes: BTreeMap<ExecutorId, OperatorInfo>,
    ) -> DataflowContext {
        DataflowContext {
            job_id,
            meta,
            nodes,
        }
    }

    fn create_source_sink(&self, source_sink_manager: &mut SourceSinkManger, executor_id: u32) {
        let operator_info = self.nodes.get(&executor_id);
        let remote_node = operator_info.filter(|operator| utils::is_remote_operator(*operator));
        let external_source = operator_info.filter(|operator| (*operator).has_source());
        let external_sink = operator_info.filter(|operator| (*operator).has_sink());

        if remote_node.is_some() {
            remote_node.iter().for_each(|operator| {
                source_sink_manager.create_remote_sink(&executor_id, *operator);
                source_sink_manager.create_empty_source(&executor_id)
            })
        } else if external_source.is_some() {
            external_source.iter().for_each(|operator| {
                source_sink_manager.create_external_source(&executor_id, *operator);
                source_sink_manager.create_empty_sink(&executor_id)
            })
        } else if external_sink.is_some() {
            external_sink.iter().for_each(|operator| {
                source_sink_manager.create_external_sink(&executor_id, *operator);
                source_sink_manager.create_empty_source(&executor_id)
            })
        } else {
            source_sink_manager.create_local(&executor_id)
        }
    }

    pub fn create_executors(&self) -> Vec<ExecutorImpl> {
        let ref mut source_sink_manager = SourceSinkManger::new(&self.job_id);
        let mut meta_map = BTreeMap::new();
        self.meta.iter().for_each(|meta| {
            meta_map.insert(meta.center, meta.neighbors.clone());

            self.create_source_sink(source_sink_manager, meta.center as ExecutorId);

            meta.neighbors.iter().for_each(|node_id| {
                let executor_id = *node_id as ExecutorId;
                self.create_source_sink(source_sink_manager, executor_id);
            })
        });

        let mut executors = vec![];

        self.meta.iter().for_each(|meta| {
            executors.push(ExecutorImpl::with_source_and_sink_config(
                &self.job_id,
                meta.center as ExecutorId,
                source_sink_manager.get_sinks_by_ids(meta.neighbors.clone()),
                source_sink_manager
                    .get_source_by_id(meta.center.clone())
                    .unwrap(),
                self.nodes.get(&meta.center).unwrap().clone(),
            ));

            meta.neighbors.iter().for_each(|id| {
                if !meta_map.contains_key(id) {
                    let neighbors = meta_map
                        .get(id)
                        .map(|slice| (*slice).to_vec())
                        .unwrap_or(vec![]);
                    executors.push(ExecutorImpl::with_source_and_sink_config(
                        &self.job_id,
                        *id as ExecutorId,
                        source_sink_manager.get_sinks_by_ids(neighbors),
                        source_sink_manager.get_source_by_id(*id).unwrap(),
                        self.nodes.get(id).unwrap().clone(),
                    ))
                }
            })
        });
        executors
    }

    pub fn validate(&self) -> bool {
        return true;
    }
}

#[async_trait]
pub trait Executor {
    async fn run(self);
    fn as_sinkable(&self) -> SinkImpl;
    fn close(&mut self);
}

pub struct LocalExecutor {
    pub job_id: ResourceId,
    pub executor_id: ExecutorId,
    pub(crate) operator: OperatorInfo,

    sinks: Vec<SinkImpl>,
    source: SourceImpl,
}

impl LocalExecutor {
    pub fn with_source_and_sink(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
        operator: OperatorInfo,
    ) -> Self {
        Self {
            job_id: job_id.clone(),
            executor_id,
            operator,
            source,
            sinks,
        }
    }

    fn process_single_event<'s, 'i, S: StateManager>(
        &self,
        task: &Execution<'s, 'i, S>,
        event: &KeyedDataEvent,
    ) {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;
        if self.operator.has_source() || self.operator.has_sink() {
            self.sinks.par_iter().for_each(|sink| {
                let _ = futures_executor::block_on(sink.sink(LocalMessage(
                    LocalEvent::KeyedDataStreamEvent(event.clone()),
                )));
            });
        } else {
            match task.process(event) {
                Ok(results) => results.iter().for_each(|event| {
                    let after_process = KeyedDataStreamEvent(event.clone());
                    self.sinks.par_iter().for_each(|sink| {
                        let result = futures_executor::block_on(
                            sink.sink(LocalMessage(after_process.clone())),
                        );
                        match result {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!("sink to node {} failed: {:?}", sink.sink_id(), err)
                            }
                        }
                    });
                }),
                Err(err) => {
                    tracing::error!("process msg failed: {:?}", err);
                    // TODO fault tolerance
                }
            }
        }
    }
}

#[async_trait]
impl Executor for LocalExecutor {
    async fn run(mut self) {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let task = Execution::new(&self.operator, new_state_mgt(&self.job_id), scope);

        loop {
            while let Some(msg) = self.source.fetch_msg() {
                match &msg {
                    LocalMessage(message) => match message {
                        KeyedDataStreamEvent(e) => self.process_single_event(&task, e),
                        LocalEvent::Terminate { job_id, to } => {
                            tracing::info!("stopping {:?} at node id {}", job_id, to);
                            // gracefully close
                            self.close();
                            return;
                        }
                    },
                }
            }
        }
    }

    fn as_sinkable(&self) -> SinkImpl {
        SinkImpl::Local(LocalSink {
            sender: self.source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }

    fn close(&mut self) {
        self.job_id.clear();
        self.operator.clear();
        drop(self.executor_id);
        self.sinks.iter_mut().for_each(|sink| sink.close_sink());
        self.sinks.clear();
        self.source.close()
    }
}

pub struct WindowExecutor {
    pub job_id: ResourceId,
    source: SourceImpl,
    executor_id: ExecutorId,
    sinks: Vec<SinkImpl>,
    assigner: WindowAssignerImpl,
    windows: VecDeque<KeyedWindow>,
}

impl WindowExecutor {
    fn with_source_and_sink(
        job_id: &ResourceId,
        executor_id: u32,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
        operator: OperatorInfo,
    ) -> Self {
        let assigner = WindowAssignerImpl::new(&operator);

        Self {
            job_id: job_id.clone(),
            source,
            executor_id,
            sinks,
            assigner,
            windows: Default::default(),
        }
    }
}

#[async_trait]
impl Executor for WindowExecutor {
    async fn run(mut self) {
        loop {
            tokio::select! {
                biased;
                _ = self.assigner.trigger() => {
                    let merged_windows = self.assigner.group_by_key_and_window(&mut self.windows);
                    self.windows.clear();
                    merged_windows.into_iter().for_each(|mut keyed_window| {
                        let event = keyed_window.as_event();
                        self.sinks.par_iter().for_each(|sink| {
                            match futures_executor::block_on(sink.sink(SinkableMessageImpl::LocalMessage(
                                LocalEvent::KeyedDataStreamEvent(event.clone()),
                            ))) {
                                Ok(_) => {}
                                Err(err) => tracing::error!("sink message failed: {:?}", err),
                            }
                        })
                    });
                },

                Some(msg) = self.source.async_fetch_msg() => {
                match msg {
                    SinkableMessageImpl::LocalMessage(event) => match event {
                        LocalEvent::Terminate { .. } => {
                            self.close();
                            return
                        },
                        LocalEvent::KeyedDataStreamEvent(keyed_event) => {
                            let windows = self.assigner.assign_windows(keyed_event);
                            self.windows.append(&mut VecDeque::from(windows));
                        },
                    },
                }
               },
            }
        }
    }

    fn as_sinkable(&self) -> SinkImpl {
        SinkImpl::Local(LocalSink {
            sender: self.source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }

    fn close(&mut self) {
        self.job_id.clear();
        drop(self.executor_id);
        self.sinks.iter_mut().for_each(|sink| sink.close_sink());
        self.sinks.clear();
        self.source.close()
    }
}

pub enum ExecutorImpl {
    Local(LocalExecutor),
    Window(WindowExecutor),
}

unsafe impl Sync for ExecutorImpl {}
unsafe impl Send for ExecutorImpl {}

impl ExecutorImpl {
    pub fn run(self) -> tokio::task::JoinHandle<()> {
        match self {
            Self::Local(exec) => tokio::spawn(exec.run()),
            Self::Window(exec) => tokio::spawn(exec.run()),
        }
    }

    pub fn as_sinkable(&self) -> SinkImpl {
        match self {
            ExecutorImpl::Local(exec) => exec.as_sinkable(),
            ExecutorImpl::Window(exec) => exec.as_sinkable(),
        }
    }

    pub fn with_source_and_sink_config(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
        operator: OperatorInfo,
    ) -> Self {
        if operator.has_window() {
            Self::Window(WindowExecutor::with_source_and_sink(
                job_id,
                executor_id,
                sinks,
                source,
                operator,
            ))
        } else {
            Self::Local(LocalExecutor::with_source_and_sink(
                job_id,
                executor_id,
                sinks,
                source,
                operator,
            ))
        }
    }
}

/// The interface for developing a source.
/// The basic [`Source`] is a stateless [`LocalSource`] that can flush data on checkpoint to achieve at-least-once consistency
#[async_trait]
pub trait Source {
    /**
     * Fetch next message
     * It may block currrent thread
     */
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl>;

    /// source id of the Source
    fn source_id(&self) -> SourceId;

    /// gracefully close this source
    fn close_source(&mut self);

    /**
     * fetch next message asynchronously
     */
    async fn async_fetch_msg(&mut self) -> Option<SinkableMessageImpl>;
}

/// Create and manager all sources and sinks for a Dataflow
pub struct SourceSinkManger {
    job_id: ResourceId,
    sources: BTreeMap<SourceId, SourceImpl>,
    sinks: BTreeMap<SinkId, SinkImpl>,
}

impl SourceSinkManger {
    pub(crate) fn get_source_by_id(&self, source_id: SourceId) -> Option<SourceImpl> {
        self.sources.get(&source_id).map(|s| s.clone())
    }

    pub(crate) fn get_sinks_by_ids(&self, sink_ids: Vec<SinkId>) -> Vec<SinkImpl> {
        self.sinks
            .iter()
            .filter(|entry| lang::any_match(&sink_ids, |id| id == entry.0))
            .map(|entry| entry.1.clone())
            .collect()
    }

    /*
    call this method will create local sink and inner source simultaneously
     */
    pub(crate) fn create_local(&mut self, executor_id: &ExecutorId) {
        if self.sources.contains_key(executor_id) {
            return;
        }
        let channel_size = get_env("CHANNEL_SIZE")
            .and_then(|size| size.parse::<usize>().ok())
            .unwrap_or(DEFAULT_CHANNEL_SIZE);
        let (tx, rx) = new_event_channel(channel_size);
        let recv = Arc::new(Mutex::new(rx));
        let source = LocalSource {
            recv,
            source_id: executor_id.clone(),
            tx: tx.clone(),
        };

        self.sources.insert(*executor_id, SourceImpl::Local(source));
        self.sinks.insert(
            *executor_id,
            SinkImpl::Local(LocalSink {
                sender: tx,
                sink_id: *executor_id,
            }),
        );
    }

    pub(crate) fn create_remote_sink(&mut self, executor_id: &ExecutorId, info: &OperatorInfo) {
        if self.sinks.contains_key(executor_id) {
            return;
        }
        let host_addr = info.get_host_addr();
        self.sinks.insert(
            *executor_id,
            SinkImpl::Remote(RemoteSink {
                sink_id: *executor_id,
                task_worker_gateway: SafeTaskManagerRpcGateway::new(&host_addr),
            }),
        );
    }

    pub(crate) fn create_external_source(
        &mut self,
        executor_id: &ExecutorId,
        operator: &OperatorInfo,
    ) {
        if self.sources.contains_key(executor_id) {
            return;
        }
        operator
            .get_source()
            .desc
            .iter()
            .for_each(|desc| match desc {
                source::Desc::Kafka(conf) => {
                    let (tx, rx) = new_event_channel(1);
                    self.sources.insert(
                        *executor_id,
                        SourceImpl::Kafka(
                            Kafka::with_source_config(&self.job_id, *executor_id, conf),
                            tx,
                            Arc::new(Mutex::new(rx)),
                        ),
                    );
                }
            })
    }

    fn create_external_sink(&mut self, executor_id: &ExecutorId, operator: &OperatorInfo) {
        if self.sinks.contains_key(executor_id) {
            return;
        }
        operator.get_sink().desc.iter().for_each(|desc| match desc {
            sink::Desc::Kafka(kafka) => {
                self.sinks.insert(
                    *executor_id,
                    SinkImpl::Kafka(Kafka::with_sink_config(&self.job_id, *executor_id, kafka)),
                );
            }
            sink::Desc::Mysql(mysql) => {
                self.sinks.insert(
                    *executor_id,
                    SinkImpl::Mysql(Mysql::with_config(*executor_id, mysql)),
                );
            }
            sink::Desc::Redis(redis) => {
                self.sinks.insert(
                    *executor_id,
                    SinkImpl::Redis(Redis::with_config(*executor_id, redis)),
                );
            }
        });
    }

    fn create_empty_source(&mut self, executor_id: &u32) {
        let (tx, rx) = new_event_channel(1);
        self.sources.insert(
            *executor_id,
            SourceImpl::Empty(*executor_id, tx, Arc::new(Mutex::new(rx))),
        );
    }

    fn create_empty_sink(&mut self, executor_id: &u32) {
        self.sinks
            .insert(*executor_id, SinkImpl::Empty(*executor_id));
    }
}

impl SourceSinkManger {
    fn new(job_id: &ResourceId) -> SourceSinkManger {
        SourceSinkManger {
            sources: Default::default(),
            sinks: Default::default(),
            job_id: job_id.clone(),
        }
    }
}

#[async_trait]
pub trait Sink {
    fn sink_id(&self) -> SinkId;

    /**
     * In streaming processing, sink should support three kinds of delivery guarantee:
     * 1. AT-LEAST-ONCE
     * 2. EXACTLY-ONCE
     * 3. NONE
     * However, if we want to make sure EXACTLY-ONCE or AT-LEAST-ONCE delivery, fault tolerance is essential.
     * In 1.0 release version, we will support checkpoint for fault tolerance and users can choose which guarantee they want Lightflus to satisfy.
     */
    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException>;

    /**
     * Gracefully close sink
     */
    fn close_sink(&mut self);
}

/// [`LocalSink`] supports transfer data between two operators in one host node.
/// In 1.0 verision, we will introduce barrier machenism and state snapshot to support checkpoint
#[derive(Clone)]
pub struct LocalSink {
    pub(crate) sender: EventSender<SinkableMessageImpl>,
    pub(crate) sink_id: SinkId,
}

#[async_trait]
impl Sink for LocalSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match &msg {
            SinkableMessageImpl::LocalMessage(_) => self
                .sender
                .send(msg)
                .await
                .map(|_| SendEventToOperatorStatusEnum::Done)
                .map_err(|err| err.into()),
        }
    }

    fn close_sink(&mut self) {
        futures_executor::block_on(self.sender.closed());
        drop(self.sink_id)
    }
}

/// [`RemoteSink`] communicates with remote worker nodes for transferring data between two hosts.
#[derive(Clone)]
pub struct RemoteSink {
    pub(crate) sink_id: SinkId,
    pub(crate) task_worker_gateway: SafeTaskManagerRpcGateway,
}

#[async_trait]
impl Sink for RemoteSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match msg {
            SinkableMessageImpl::LocalMessage(event) => match event {
                LocalEvent::Terminate { .. } => Ok(SendEventToOperatorStatusEnum::Done),
                LocalEvent::KeyedDataStreamEvent(e) => self
                    .task_worker_gateway
                    .send_event_to_operator(e)
                    .await
                    .map(|resp| resp.status())
                    .map_err(|err| SinkException {
                        kind: RemoteSinkFailed,
                        msg: format!("{}", err),
                    }),
            },
        }
    }

    fn close_sink(&mut self) {
        drop(self.sink_id);
        self.task_worker_gateway.close()
    }
}

#[derive(Clone)]
pub struct LocalSource {
    pub(crate) recv: Arc<Mutex<EventReceiver<SinkableMessageImpl>>>,
    pub(crate) source_id: SourceId,
    tx: EventSender<SinkableMessageImpl>,
}

#[async_trait]
impl Source for LocalSource {
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        futures_executor::block_on(self.async_fetch_msg())
    }

    fn source_id(&self) -> SourceId {
        self.source_id
    }

    fn close_source(&mut self) {
        futures_executor::block_on(async {
            self.recv.lock().await.close();
            self.tx.closed().await
        });
        drop(self.source_id)
    }

    async fn async_fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        self.recv.lock().await.recv().await
    }
}

impl LocalSource {
    pub fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        self.tx.clone()
    }
}

#[derive(Clone)]
pub enum SourceImpl {
    Local(LocalSource),
    Kafka(
        Kafka,
        EventSender<SinkableMessageImpl>,
        Arc<Mutex<EventReceiver<SinkableMessageImpl>>>,
    ),
    Empty(
        SourceId,
        EventSender<SinkableMessageImpl>,
        Arc<Mutex<EventReceiver<SinkableMessageImpl>>>,
    ),
}

impl SourceImpl {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        match self {
            Self::Local(source) => source.create_msg_sender(),
            Self::Kafka(.., terminator_tx, _) => terminator_tx.clone(),
            Self::Empty(.., terminator_tx, _) => terminator_tx.clone(),
        }
    }

    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        futures_executor::block_on(self.async_fetch_msg())
    }

    async fn async_fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        match self {
            Self::Local(source) => source.async_fetch_msg().await,
            Self::Kafka(source, _, terminator_rx) => {
                match terminator_rx.lock().await.try_recv() {
                    Ok(message) => Some(message),
                    Err(err) => match err {
                        // if channel has been close, it will terminate all actors
                        TryRecvError::Disconnected => {
                            Some(SinkableMessageImpl::LocalMessage(LocalEvent::Terminate {
                                job_id: source.job_id.clone(),
                                to: source.source_id(),
                            }))
                        }
                        _ => source.async_fetch_msg().await,
                    },
                }
            }
            Self::Empty(.., terminator_rx) => terminator_rx.lock().await.recv().await,
        }
    }

    fn close(&mut self) {
        match self {
            Self::Local(source) => source.close_source(),
            Self::Kafka(kafka, tx, rx) => {
                kafka.close_source();
                futures_executor::block_on(async {
                    rx.lock().await.close();
                    tx.closed().await;
                });
            }
            Self::Empty(id, tx, rx) => {
                drop(id);
                futures_executor::block_on(async {
                    rx.lock().await.close();
                    tx.closed().await;
                });
            }
        }
    }
}

#[derive(Clone)]
pub enum SinkImpl {
    Local(LocalSink),
    Remote(RemoteSink),
    Kafka(Kafka),
    Mysql(Mysql),
    Redis(Redis),
    Empty(SinkId),
}

#[async_trait]
impl Sink for SinkImpl {
    fn sink_id(&self) -> SinkId {
        match self {
            Self::Local(sink) => sink.sink_id(),
            Self::Remote(sink) => sink.sink_id(),
            Self::Kafka(kafka) => kafka.sink_id(),
            Self::Mysql(mysql) => mysql.sink_id(),
            Self::Empty(sink_id) => *sink_id,
            Self::Redis(redis) => redis.sink_id(),
        }
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match self {
            Self::Local(sink) => sink.sink(msg).await,
            Self::Remote(sink) => sink.sink(msg).await,
            Self::Kafka(sink) => sink.sink(msg).await,
            Self::Mysql(sink) => sink.sink(msg).await,
            Self::Empty(_) => Ok(SendEventToOperatorStatusEnum::Done),
            Self::Redis(redis) => redis.sink(msg).await,
        }
    }

    fn close_sink(&mut self) {
        match self {
            Self::Local(sink) => sink.close_sink(),
            Self::Remote(sink) => sink.close_sink(),
            Self::Kafka(sink) => sink.close_sink(),
            Self::Mysql(sink) => sink.close_sink(),
            Self::Redis(sink) => sink.close_sink(),
            Self::Empty(id) => drop(id),
        }
    }
}

/// An unified implementation for Kafka Source and Sink.
#[derive(Clone)]
pub struct Kafka {
    connector_id: SourceId,
    conf: KafkaDesc,
    job_id: ResourceId,
    consumer: Option<Arc<KafkaConsumer>>,
    producer: Option<KafkaProducer>,
}

impl Kafka {
    pub fn with_source_config(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        config: &KafkaDesc,
    ) -> Kafka {
        let mut self_ = Kafka {
            connector_id: executor_id,
            conf: config.clone(),
            job_id: job_id.clone(),
            consumer: None,
            producer: None,
        };
        match run_consumer(
            config
                .brokers
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<String>>()
                .join(",")
                .as_str(),
            &config.get_kafka_group(),
            &config.topic,
        ) {
            Ok(consumer) => self_.consumer = Some(Arc::new(consumer)),
            Err(err) => tracing::error!("kafka source connect failed: {}", err),
        };

        self_
    }

    pub fn with_sink_config(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        config: &KafkaDesc,
    ) -> Kafka {
        let mut self_ = Kafka {
            connector_id: executor_id,
            conf: config.clone(),
            job_id: job_id.clone(),
            consumer: None,
            producer: None,
        };
        match run_producer(
            config
                .brokers
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<String>>()
                .join(",")
                .as_str(),
            &config.topic,
            &config.get_kafka_group(),
            config.get_kafka_partition() as i32,
        ) {
            Ok(producer) => self_.producer = Some(producer),
            Err(err) => tracing::error!("kafka producer create failed: {}", err),
        }

        self_
    }

    fn process(&self, message: KafkaMessage) -> SinkableMessageImpl {
        let data_type = self.conf.data_type();
        let payload = message.payload.as_slice();
        let key = TypedValue::from_vec(&message.key);
        let val = TypedValue::from_slice_with_type(payload, data_type);

        let result =
            SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
                job_id: Some(self.job_id.clone()),
                key: Some(Entry {
                    data_type: key.get_type() as i32,
                    value: key.get_data(),
                }),
                to_operator_id: 0,
                data: vec![Entry {
                    data_type: self.conf.data_type() as i32,
                    value: val.get_data(),
                }],

                event_time: message
                    .timestamp
                    .map(|millis| Duration::from_millis(millis as u64))
                    .and_then(|duration| {
                        SystemTime::UNIX_EPOCH
                            .checked_add(duration)
                            .map(|time| chrono::DateTime::<chrono::Utc>::from(time))
                    })
                    .map(|time| Timestamp {
                        seconds: time.naive_utc().timestamp(),
                        nanos: time.naive_utc().timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: self.connector_id,
                window: None,
            }));

        result
    }
}

#[async_trait]
impl Source for Kafka {
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        futures_executor::block_on(self.async_fetch_msg())
    }

    fn source_id(&self) -> SourceId {
        self.connector_id
    }

    fn close_source(&mut self) {
        self.conf.clear();
        self.job_id.clear();
        drop(self.connector_id);
        self.consumer.iter().for_each(|consumer| {
            consumer.unsubscribe();
            drop(consumer)
        })
    }

    async fn async_fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        match &self.consumer {
            Some(consumer) => consumer.fetch(|message| self.process(message)).await,
            None => None,
        }
    }
}

#[async_trait::async_trait]
impl Sink for Kafka {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        match &self.producer {
            Some(producer) => {
                let result = msg.get_kafka_message();
                match result.map_err(|err| err.into()) {
                    Ok(messages) => {
                        for msg in messages {
                            let send_result = producer.send(&msg.key, &msg.payload).await;
                            if send_result.is_err() {
                                return send_result
                                    .map(|_| SendEventToOperatorStatusEnum::Failure)
                                    .map_err(|err| err.into());
                            }
                        }

                        Ok(SendEventToOperatorStatusEnum::Done)
                    }
                    Err(err) => Err(err),
                }
            }
            None => Ok(SendEventToOperatorStatusEnum::Done),
        }
    }

    fn close_sink(&mut self) {
        drop(self.connector_id);
        self.conf.clear();
        self.job_id.clear();
        self.producer
            .iter_mut()
            .for_each(|producer| producer.close())
    }
}

/// An unified implementation for Mysql Source and Sink
#[derive(Clone)]
pub struct Mysql {
    connector_id: SinkId,
    statement: String,
    extractors: Vec<String>,
    conn: MysqlConn,
}
impl Mysql {
    pub fn with_config(connector_id: u32, conf: &MysqlDesc) -> Mysql {
        let mut statement = conf.get_mysql_statement();
        statement
            .extractors
            .sort_by(|v1, v2| v1.index.cmp(&v2.index));
        let extractors = statement
            .extractors
            .iter()
            .map(|e| e.extractor.clone())
            .collect();

        let statement = statement.statement;

        let connection_opts = conf
            .connection_opts
            .as_ref()
            .map(|opts| opts.clone())
            .unwrap_or_default();

        let conn = MysqlConn::from(connection_opts);

        Mysql {
            connector_id,
            statement,
            extractors,
            conn,
        }
    }

    fn get_arguments(&self, msg: SinkableMessageImpl) -> Vec<Vec<TypedValue>> {
        extract_arguments(self.extractors.as_slice(), msg, "mysql_extractor")
    }
}

#[async_trait]
impl Sink for Mysql {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        let row_arguments = self.get_arguments(msg);
        let ref mut conn_result = self.conn.connect().await;
        match conn_result {
            Ok(conn) => {
                for arguments in row_arguments {
                    let result = self
                        .conn
                        .execute(&self.statement, arguments, conn)
                        .await
                        .map_err(|err| err.into());
                    if result.is_err() {
                        return result.map(|_| SendEventToOperatorStatusEnum::Failure);
                    }
                }

                Ok(SendEventToOperatorStatusEnum::Done)
            }
            Err(err) => Err(err.into()),
        }
    }

    fn close_sink(&mut self) {
        self.conn.close();
        self.extractors.clear();
        drop(self.connector_id);
        self.statement.clear();
    }
}

/// An unified implement for Redis Source and Sink
#[derive(Clone)]
pub struct Redis {
    connector_id: SinkId,
    key_extractor: String,
    value_extractor: String,
    client: RedisClient,
}

impl Redis {
    pub fn with_config(connector_id: SinkId, conf: &RedisDesc) -> Self {
        let client = RedisClient::new(&conf);
        let key_extractor = conf
            .key_extractor
            .as_ref()
            .map(|func| func.function.clone())
            .unwrap_or_default();
        let value_extractor = conf
            .value_extractor
            .as_ref()
            .map(|func| func.function.clone())
            .unwrap_or_default();
        Self {
            connector_id,
            key_extractor,
            value_extractor,
            client,
        }
    }
}

#[async_trait::async_trait]
impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<SendEventToOperatorStatusEnum, SinkException> {
        let key_values = extract_arguments(
            &[self.key_extractor.clone(), self.value_extractor.clone()],
            msg.clone(),
            "redis_extractor",
        );
        let mut conn_result = self.client.connect();
        conn_result
            .as_mut()
            .map_err(|err| err.into())
            .and_then(|conn| {
                let kvs = Vec::from_iter(key_values.iter().map(|kv| (&kv[0], &kv[1])));
                self.client
                    .set_multiple(conn, kvs.as_slice())
                    .map(|_| SendEventToOperatorStatusEnum::Done)
                    .map_err(|err| err.into())
            })
    }

    fn close_sink(&mut self) {
        drop(self.connector_id);
        self.key_extractor.clear();
        self.value_extractor.clear();
    }
}

fn extract_arguments(
    extractors: &[String],
    message: SinkableMessageImpl,
    fn_name: &str,
) -> Vec<Vec<TypedValue>> {
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    match message {
        SinkableMessageImpl::LocalMessage(event) => match event {
            LocalEvent::Terminate { .. } => vec![],
            LocalEvent::KeyedDataStreamEvent(e) => Vec::from_iter(e.data.iter().map(|entry| {
                let val = TypedValue::from_slice(&entry.value);
                extractors
                    .iter()
                    .map(|extractor| {
                        let mut rt_engine = RuntimeEngine::new(extractor.as_str(), fn_name, scope);
                        rt_engine.call_one_arg(&val).unwrap_or_default()
                    })
                    .collect::<Vec<TypedValue>>()
            })),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common::{
        event::{LocalEvent, SinkableMessageImpl},
        utils::times::from_millis_to_utc_chrono,
    };
    use prost_types::Timestamp;
    use proto::common::{
        keyed_data_event, mysql_desc, operator_info::Details, redis_desc, sink, trigger, window,
        Entry, Func, KafkaDesc, KeyedDataEvent, MysqlDesc, OperatorInfo, RedisDesc, ResourceId,
        Time, Trigger, Window,
    };
    use tokio::sync::Mutex;

    use crate::{
        actor::{Sink, SinkImpl, SourceImpl},
        new_event_channel,
        window::KeyedWindow,
    };

    struct SetupGuard {}

    impl Drop for SetupGuard {
        fn drop(&mut self) {}
    }

    fn setup_v8() -> SetupGuard {
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
    pub fn test_new_dataflow_context() {
        use proto::common::DataflowMeta;
        use proto::common::OperatorInfo;
        use proto::common::ResourceId;
        use proto::common::{Filter, MysqlDesc, Sink};
        use std::collections::BTreeMap;

        use crate::actor::DataflowContext;

        let mut metas = vec![];
        let mut meta = DataflowMeta::default();
        meta.center = 0;
        meta.neighbors = vec![1];

        metas.push(meta);
        let mut meta = DataflowMeta::default();
        meta.center = 1;
        meta.neighbors = vec![2];

        metas.push(meta);

        let mut nodes = BTreeMap::new();
        let mut node_1 = OperatorInfo::default();
        node_1.operator_id = 0;

        nodes.insert(0, node_1);

        let mut op_1 = OperatorInfo::default();
        op_1.operator_id = 1;
        op_1.details = Some(Details::Filter(Filter::default()));
        nodes.insert(1, op_1);

        let mysql = OperatorInfo {
            operator_id: 2,
            host_addr: None,
            upstreams: vec![1],
            details: Some(Details::Sink(Sink {
                desc: Some(sink::Desc::Mysql(MysqlDesc {
                    connection_opts: Some(mysql_desc::ConnectionOpts {
                        host: "localhost".to_string(),
                        username: "root".to_string(),
                        password: "123".to_string(),
                        database: "test".to_string(),
                    }),
                    statement: Some(mysql_desc::Statement {
                        statement: "select".to_string(),
                        extractors: vec![],
                    }),
                })),
            })),
        };
        nodes.insert(2, mysql);

        let ctx = DataflowContext::new(ResourceId::default(), metas, nodes);
        let executors = ctx.create_executors();
        assert_eq!(executors.len(), 3);
    }

    #[test]
    pub fn test_get_mysql_arguments() {
        use proto::common::MysqlDesc;

        use super::Mysql;
        use common::event::LocalEvent;
        use proto::common::KeyedDataEvent;

        use crate::actor::SinkableMessageImpl;
        use std::collections::BTreeMap;

        use common::types::TypedValue;

        let _setup_guard = setup_v8();

        let desc = MysqlDesc {
            connection_opts: Some(mysql_desc::ConnectionOpts {
                host: "localhost".to_string(),
                username: "root".to_string(),
                password: "123".to_string(),
                database: "test".to_string(),
            }),
            statement: Some(mysql_desc::Statement {
                statement: "INSERT INTO table VALUES (?, ?)".to_string(),
                extractors: vec![
                    mysql_desc::statement::Extractor {
                        index: 1,
                        extractor: "function mysql_extractor(a) {return a.v1}".to_string(),
                    },
                    mysql_desc::statement::Extractor {
                        index: 2,
                        extractor: "function mysql_extractor(a) {return a.v2}".to_string(),
                    },
                ],
            }),
        };

        let mysql = Mysql::with_config(1, &desc);
        let mut event = KeyedDataEvent::default();
        let mut entry_1 = BTreeMap::new();
        [
            ("v1", TypedValue::Number(1.0)),
            ("v2", TypedValue::String("value".to_string())),
        ]
        .iter()
        .for_each(|pair| {
            entry_1.insert(pair.0.to_string(), pair.1.clone());
        });
        event.data = Vec::from_iter([TypedValue::Object(entry_1)].iter().map(|value| Entry {
            data_type: value.get_type() as i32,
            value: value.get_data(),
        }));
        let message = SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(event));
        let arguments = mysql.get_arguments(message);
        assert_eq!(
            arguments,
            vec![vec![
                TypedValue::Number(1.0),
                TypedValue::String("value".to_string())
            ]]
        )
    }

    #[tokio::test]
    async fn test_kafka_source_sink_close() {
        let job_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "ns_id".to_string(),
        };
        let desc = KafkaDesc {
            brokers: vec!["localhost:9092".to_string()],
            topic: "topic".to_string(),
            opts: None,
            data_type: 6,
        };
        let (tx, rx) = new_event_channel(1);
        let mut kafka_source = SourceImpl::Kafka(
            super::Kafka::with_source_config(&job_id, 0, &desc),
            tx.clone(),
            Arc::new(Mutex::new(rx)),
        );

        let mut kafka_sink = SinkImpl::Kafka(super::Kafka::with_sink_config(&job_id, 0, &desc));

        kafka_source.close();

        match &mut kafka_source {
            SourceImpl::Kafka(kafka, tx, rx) => {
                assert_eq!(&kafka.job_id, &ResourceId::default());
                assert_eq!(
                    &kafka.conf,
                    &KafkaDesc {
                        brokers: vec![],
                        topic: Default::default(),
                        opts: None,
                        data_type: 0,
                    }
                );
                assert!(tx.is_closed());
                let result = rx.lock().await.try_recv();
                assert!(result.is_err())
            }
            _ => {}
        }

        kafka_sink.close_sink();

        match &kafka_sink {
            SinkImpl::Kafka(kafka) => {
                assert_eq!(&kafka.job_id, &ResourceId::default());
                assert_eq!(
                    &kafka.conf,
                    &KafkaDesc {
                        brokers: vec![],
                        topic: Default::default(),
                        opts: None,
                        data_type: 0,
                    }
                );
            }
            _ => {}
        }
    }

    #[test]
    fn test_redis_source_sink_close() {
        let desc = RedisDesc {
            connection_opts: Some(redis_desc::ConnectionOpts {
                host: "localhost".to_string(),
                username: Default::default(),
                password: Default::default(),
                database: 0,
                tls: false,
            }),
            key_extractor: Some(Func {
                function: "key_extractor".to_string(),
            }),
            value_extractor: Some(Func {
                function: "value_extractor".to_string(),
            }),
        };
        let mut redis_sink = SinkImpl::Redis(super::Redis::with_config(0, &desc));

        redis_sink.close_sink();
        match redis_sink {
            SinkImpl::Redis(redis) => {
                assert_eq!(&redis.key_extractor, "");
                assert_eq!(&redis.value_extractor, "");
            }
            _ => {}
        }
    }

    #[test]
    fn test_mysql_sink_close() {
        let ref conf = MysqlDesc {
            connection_opts: Some(mysql_desc::ConnectionOpts {
                host: "localhost".to_string(),
                username: "root".to_string(),
                password: "123".to_string(),
                database: "test".to_string(),
            }),
            statement: Some(mysql_desc::Statement {
                statement: "statement".to_string(),
                extractors: vec![mysql_desc::statement::Extractor {
                    index: 0,
                    extractor: "extrator".to_string(),
                }],
            }),
        };
        let mut mysql_sink = SinkImpl::Mysql(super::Mysql::with_config(0, conf));
        mysql_sink.close_sink();
        match mysql_sink {
            SinkImpl::Mysql(mysql) => {
                assert!(mysql.extractors.is_empty());
                assert!(mysql.statement.is_empty());
            }
            _ => {}
        }
    }

    #[tokio::test]
    async fn test_local_source_sink_close() {
        {
            let (tx, rx) = new_event_channel(1);
            let mut source = SourceImpl::Local(super::LocalSource {
                recv: Arc::new(Mutex::new(rx)),
                source_id: 0,
                tx: tx.clone(),
            });

            source.close();

            assert!(source.fetch_msg().is_none());
            assert!(tx.is_closed());
        }

        {
            let (tx, mut rx) = new_event_channel(10);
            let mut sink = SinkImpl::Local(super::LocalSink {
                sender: tx,
                sink_id: 0,
            });
            rx.close();

            sink.close_sink();
            let result = sink
                .sink(SinkableMessageImpl::LocalMessage(LocalEvent::Terminate {
                    job_id: ResourceId::default(),
                    to: 0,
                }))
                .await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_actor_close() {
        use std::time::Duration;

        use common::event::{LocalEvent, SinkableMessageImpl};
        use proto::common::{DataTypeEnum, KafkaDesc, OperatorInfo, ResourceId};
        use stream::actor::{ExecutorImpl, Kafka, LocalExecutor, SinkImpl, SourceImpl};
        let _ = setup_v8();
        let job_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "ns_id".to_string(),
        };
        let (tx, rx) = new_event_channel(1);
        let innert_tx = tx.clone();
        let source = SourceImpl::Empty(0, tx, Arc::new(Mutex::new(rx)));
        let sinks = vec![SinkImpl::Kafka(Kafka::with_sink_config(
            &job_id,
            1,
            &KafkaDesc {
                brokers: vec!["localhost:9092".to_string()],
                topic: "topic".to_string(),
                opts: None,
                data_type: DataTypeEnum::Object as i32,
            },
        ))];
        let operator = OperatorInfo::default();
        let executor = LocalExecutor::with_source_and_sink(&job_id, 0, sinks, source, operator);
        let executor = ExecutorImpl::Local(executor);
        let handler = tokio::spawn(async { executor.run() });
        let result = innert_tx
            .send(SinkableMessageImpl::LocalMessage(LocalEvent::Terminate {
                job_id,
                to: 0,
            }))
            .await;
        assert!(result.is_ok());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(handler.is_finished())
    }

    #[tokio::test]
    async fn test_window_executor_run() {
        let job_id = &ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "nsid".to_string(),
        };
        let (sender, mut rx) = new_event_channel(10);

        let sink = SinkImpl::Local(super::LocalSink { sender, sink_id: 2 });

        let (tx, recv) = new_event_channel(10);

        let source = SourceImpl::Local(super::LocalSource {
            recv: Arc::new(Mutex::new(recv)),
            source_id: 0,
            tx,
        });

        let tx = source.create_msg_sender();
        let executor = super::ExecutorImpl::with_source_and_sink_config(
            job_id,
            1,
            vec![sink],
            source,
            OperatorInfo {
                operator_id: 1,
                host_addr: None,
                upstreams: vec![0],
                details: Some(Details::Window(Window {
                    trigger: Some(Trigger {
                        value: Some(trigger::Value::Watermark(trigger::Watermark {
                            trigger_time: Some(Time {
                                millis: 350,
                                seconds: 0,
                                minutes: 0,
                                hours: 0,
                            }),
                        })),
                    }),
                    value: Some(window::Value::Fixed(window::FixedWindow {
                        size: Some(Time {
                            millis: 300,
                            seconds: 0,
                            minutes: 0,
                            hours: 0,
                        }),
                    })),
                })),
            },
        );

        let handler = executor.run();
        let event_time_0 = chrono::Utc::now();
        let r = tx
            .send(SinkableMessageImpl::LocalMessage(
                LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
                    job_id: Some(job_id.clone()),
                    key: None,
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![2],
                    }],
                    event_time: Some(Timestamp {
                        seconds: event_time_0.timestamp(),
                        nanos: event_time_0.timestamp_subsec_nanos() as i32,
                    }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                }),
            ))
            .await;
        assert!(r.is_ok());
        let duration = std::time::Duration::from_millis(150);

        tokio::time::sleep(duration).await;

        let event_time_1 = chrono::Utc::now();
        let r = tx
            .send(SinkableMessageImpl::LocalMessage(
                LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
                    job_id: Some(job_id.clone()),
                    key: None,
                    to_operator_id: 1,
                    data: vec![Entry {
                        data_type: 1,
                        value: vec![3],
                    }],
                    event_time: Some(Timestamp {
                        seconds: event_time_1.timestamp(),
                        nanos: event_time_1.timestamp_subsec_nanos() as i32,
                    }),
                    process_time: None,
                    from_operator_id: 0,
                    window: None,
                }),
            ))
            .await;
        assert!(r.is_ok());
        let duration = std::time::Duration::from_millis(250);

        tokio::time::sleep(duration).await;

        let window_start =
            KeyedWindow::get_window_start_with_offset(event_time_0.timestamp_millis(), 0, 300);
        let window_start = from_millis_to_utc_chrono(window_start);
        let window_start_1 =
            KeyedWindow::get_window_start_with_offset(event_time_1.timestamp_millis(), 0, 300);
        let window_start_1 = from_millis_to_utc_chrono(window_start_1);

        if window_start == window_start_1 {
            let message = rx.recv().await;
            assert!(message.is_some());
            let message = message.unwrap();

            assert_eq!(
                message,
                SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(
                    KeyedDataEvent {
                        job_id: Some(job_id.clone()),
                        key: None,
                        to_operator_id: 1,
                        data: vec![
                            Entry {
                                data_type: 1,
                                value: vec![3],
                            },
                            Entry {
                                data_type: 1,
                                value: vec![2],
                            }
                        ],
                        event_time: window_start.as_ref().map(|event_time| Timestamp {
                            seconds: event_time.timestamp(),
                            nanos: event_time.timestamp_subsec_nanos() as i32,
                        }),
                        process_time: None,
                        from_operator_id: 0,
                        window: Some(keyed_data_event::Window {
                            start_time: window_start.as_ref().map(|event_time| Timestamp {
                                seconds: event_time.timestamp(),
                                nanos: event_time.timestamp_subsec_nanos() as i32,
                            }),
                            end_time: window_start
                                .as_ref()
                                .and_then(|start| start
                                    .checked_add_signed(chrono::Duration::milliseconds(300)))
                                .map(|event_time| Timestamp {
                                    seconds: event_time.timestamp(),
                                    nanos: event_time.timestamp_subsec_nanos() as i32,
                                })
                        }),
                    }
                ))
            );
        } else {
            let message = rx.recv().await;
            assert!(message.is_some());
            let message = message.unwrap();

            assert_eq!(
                message,
                SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(
                    KeyedDataEvent {
                        job_id: Some(job_id.clone()),
                        key: None,
                        to_operator_id: 1,
                        data: vec![Entry {
                            data_type: 1,
                            value: vec![2],
                        }],
                        event_time: window_start.as_ref().map(|event_time| Timestamp {
                            seconds: event_time.timestamp(),
                            nanos: event_time.timestamp_subsec_nanos() as i32,
                        }),
                        process_time: None,
                        from_operator_id: 0,
                        window: Some(keyed_data_event::Window {
                            start_time: window_start.as_ref().map(|event_time| Timestamp {
                                seconds: event_time.timestamp(),
                                nanos: event_time.timestamp_subsec_nanos() as i32,
                            }),
                            end_time: window_start
                                .as_ref()
                                .and_then(|start| start
                                    .checked_add_signed(chrono::Duration::milliseconds(300)))
                                .map(|event_time| Timestamp {
                                    seconds: event_time.timestamp(),
                                    nanos: event_time.timestamp_subsec_nanos() as i32,
                                })
                        }),
                    }
                ))
            );

            let message = rx.recv().await;
            assert!(message.is_some());
            let message = message.unwrap();

            assert_eq!(
                message,
                SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(
                    KeyedDataEvent {
                        job_id: Some(job_id.clone()),
                        key: None,
                        to_operator_id: 1,
                        data: vec![Entry {
                            data_type: 1,
                            value: vec![3],
                        }],
                        event_time: window_start_1.as_ref().map(|event_time| Timestamp {
                            seconds: event_time.timestamp(),
                            nanos: event_time.timestamp_subsec_nanos() as i32,
                        }),
                        process_time: None,
                        from_operator_id: 0,
                        window: Some(keyed_data_event::Window {
                            start_time: window_start_1.as_ref().map(|event_time| Timestamp {
                                seconds: event_time.timestamp(),
                                nanos: event_time.timestamp_subsec_nanos() as i32,
                            }),
                            end_time: window_start_1
                                .as_ref()
                                .and_then(|start| start
                                    .checked_add_signed(chrono::Duration::milliseconds(300)))
                                .map(|event_time| Timestamp {
                                    seconds: event_time.timestamp(),
                                    nanos: event_time.timestamp_subsec_nanos() as i32,
                                })
                        }),
                    }
                ))
            );
        }

        handler.abort();
    }
}

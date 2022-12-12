use crate::dataflow::{DataflowTask, KeyedWindow, WindowAssignerImpl};
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::{new_state_mgt, StateManager};
use crate::v8_runtime::RuntimeEngine;
use crate::{EventReceiver, EventSender, DEFAULT_CHANNEL_SIZE};

use async_trait::async_trait;
use common::collections::lang;
use common::db::MysqlConn;
use common::event::{LocalEvent, SinkableMessage, SinkableMessageImpl};
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer};

use common::redis::RedisClient;
use common::types::{ExecutorId, SinkId, SourceId, TypedValue};
use common::utils::{self, get_env};
use prost::Message;
use prost_types::Timestamp;

use proto::common::{sink, source, DataflowMeta, KafkaDesc, MysqlDesc, OperatorInfo, RedisDesc};
use proto::common::{Entry, KeyedDataEvent};
use proto::common::{HostAddr, ResourceId};
use proto::worker::task_worker_api_client::TaskWorkerApiClient;
use proto::worker::{DispatchDataEventStatusEnum, DispatchDataEventsRequest, StopDataflowRequest};

use std::collections::{BTreeMap, VecDeque};

use std::rc::Rc;

use rayon::prelude::*;
use std::sync::Arc;
use std::time::SystemTime;
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

pub trait Executor {
    fn run(&mut self);
    fn as_sinkable(&self) -> SinkImpl;
    fn close(&mut self);
}

#[derive(Clone)]
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
        task: &DataflowTask<'s, 'i, S>,
        event: &KeyedDataEvent,
    ) {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;

        match task.process(event) {
            Ok(results) => results.iter().for_each(|event| {
                let after_process = KeyedDataStreamEvent(event.clone());
                self.sinks.par_iter().for_each(|sink| {
                    let result =
                        futures_executor::block_on(sink.sink(LocalMessage(after_process.clone())));
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

impl Executor for LocalExecutor {
    fn run(&mut self) {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let task = DataflowTask::new(&self.operator, new_state_mgt(&self.job_id), scope);
        loop {
            while let Some(msg) = self.source.fetch_msg() {
                match &msg {
                    LocalMessage(message) => match message {
                        KeyedDataStreamEvent(e) => {
                            if self.operator.has_source() || self.operator.has_sink() {
                                self.sinks.par_iter().for_each(|sink| {
                                    let _ = futures_executor::block_on(sink.sink(msg.clone()));
                                });
                            } else {
                                match task.process(e) {
                                    Ok(results) => results.iter().for_each(|event| {
                                        let after_process = KeyedDataStreamEvent(event.clone());
                                        self.sinks.par_iter().for_each(|sink| {
                                            let result = futures_executor::block_on(
                                                sink.sink(LocalMessage(after_process.clone())),
                                            );
                                            match result {
                                                Ok(_) => {}
                                                Err(err) => tracing::error!(
                                                    "sink to node {} failed: {:?}",
                                                    sink.sink_id(),
                                                    err
                                                ),
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

#[derive(Clone)]
pub struct WindowExecutor {
    pub job_id: ResourceId,
    source: SourceImpl,
    executor_id: ExecutorId,
    sinks: Vec<SinkImpl>,
    assigner: WindowAssignerImpl,
    windows: RefCell<VecDeque<KeyedWindow>>,
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

impl Executor for WindowExecutor {
    fn run(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            'outside: loop {
                tokio::select! {
                    biased;
                    _ = self.assigner.trigger() => {
                        RefMut::map(self.windows.borrow_mut(), |keyed_windows| {
                            let ref mut merged_windows = self.assigner.group_by_key_and_window(keyed_windows);
                            keyed_windows.clear();
                            merged_windows
                                .par_iter_mut()
                                .map(|keyed_window| keyed_window.as_event())
                                .for_each(|event| {
                                    self.sinks.par_iter().for_each(|sink| {
                                        match futures_executor::block_on(sink.sink(
                                            SinkableMessageImpl::LocalMessage(
                                                LocalEvent::KeyedDataStreamEvent(event.clone()),
                                            ),
                                        )) {
                                            Ok(_) => {}
                                            Err(err) => tracing::error!("sink message failed: {:?}", err),
                                        }
                                    })
                                });
                            keyed_windows
                        });
                    },

                    Some(msg) = async { self.source.fetch_msg() } => {
                    match msg {
                        SinkableMessageImpl::LocalMessage(event) => match event {
                            LocalEvent::Terminate { job_id, to } => break 'outside,
                            LocalEvent::KeyedDataStreamEvent(keyed_event) => {
                                let windows = self.assigner.assign_windows(&keyed_event);
                                let mut keyed_windows = self.windows.borrow_mut();
                                keyed_windows.extend(windows);
                            },
                        },
                    }
                   },
                   else => continue
                }
            }
        })
    }

    fn as_sinkable(&self) -> SinkImpl {
        SinkImpl::Local(LocalSink {
            sender: self.source.create_msg_sender(),
            sink_id: self.executor_id as u32,
        })
    }
}

#[derive(Clone)]
pub enum ExecutorImpl {
    Local(LocalExecutor),
    Window(WindowExecutor),
}

unsafe impl Send for ExecutorImpl {}
unsafe impl Sync for ExecutorImpl {}

impl ExecutorImpl {
    pub fn run(self) {
        match self {
            ExecutorImpl::Window(exec) => exec.run(),
            ExecutorImpl::Local(mut exec) => exec.run(),
        }
    }

    pub fn as_sinkable(&self) -> SinkImpl {
        match self {
            ExecutorImpl::Local(exec) => exec.as_sinkable(),
            ExecutorImpl::Window(exec) => exec.as_sinkable(),
        }
    }

    fn with_source_and_sink_config(
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

/*
Source Interface. Each Source should implement it
 */
pub trait Source {
    /**
     * Fetch next message
     * It may block currrent thread
     */
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl>;
    fn source_id(&self) -> SourceId;
    fn close_source(&mut self);
}

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
        let (tx, rx) = tokio::sync::mpsc::channel(channel_size);
        let recv = Rc::new(rx);
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
        self.sinks.insert(
            *executor_id,
            SinkImpl::Remote(RemoteSink {
                sink_id: *executor_id,
                host_addr: info.get_host_addr(),
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
                    let (tx, rx) = tokio::sync::mpsc::channel(1);
                    self.sources.insert(
                        *executor_id,
                        SourceImpl::Kafka(
                            Kafka::with_source_config(&self.job_id, *executor_id, conf),
                            tx,
                            Rc::new(rx),
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
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.sources.insert(
            *executor_id,
            SourceImpl::Empty(*executor_id, tx, Rc::new(rx)),
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
    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException>;
    fn close_sink(&mut self);
}

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
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match &msg {
            SinkableMessageImpl::LocalMessage(_) => self
                .sender
                .send(msg)
                .await
                .map(|_| DispatchDataEventStatusEnum::Done)
                .map_err(|err| err.into()),
        }
    }

    fn close_sink(&mut self) {
        futures_executor::block_on(self.sender.closed());
        drop(self.sink_id)
    }
}

#[derive(Clone)]
pub struct RemoteSink {
    pub(crate) sink_id: SinkId,
    pub(crate) host_addr: HostAddr,
}

#[async_trait]
impl Sink for RemoteSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
        let ref mut result = TaskWorkerApiClient::connect(format!(
            "{}:{}",
            &self.host_addr.host, self.host_addr.port
        ))
        .await;

        match result.as_mut().map_err(|err| err.into()) {
            Ok(cli) => match msg {
                SinkableMessageImpl::LocalMessage(event) => match event {
                    LocalEvent::Terminate { job_id, .. } => {
                        let req = StopDataflowRequest {
                            job_id: Some(job_id),
                        };
                        cli.stop_dataflow(tonic::Request::new(req))
                            .await
                            .map(|_| DispatchDataEventStatusEnum::Dispatching)
                            .map_err(|err| SinkException {
                                kind: RemoteSinkFailed,
                                msg: format!("{}", err),
                            })
                    }
                    LocalEvent::KeyedDataStreamEvent(event) => {
                        let req = DispatchDataEventsRequest {
                            events: vec![event],
                        };
                        cli.dispatch_data_events(tonic::Request::new(req))
                            .await
                            .map(|resp| {
                                for key in resp.get_ref().status_set.keys() {
                                    match resp.get_ref().get_status_set(&key).unwrap_or_default() {
                                        DispatchDataEventStatusEnum::Dispatching => {
                                            return DispatchDataEventStatusEnum::Dispatching
                                        }
                                        DispatchDataEventStatusEnum::Done => continue,
                                        DispatchDataEventStatusEnum::Failure => {
                                            return DispatchDataEventStatusEnum::Failure
                                        }
                                    }
                                }

                                DispatchDataEventStatusEnum::Done
                            })
                            .map_err(|err| SinkException {
                                kind: RemoteSinkFailed,
                                msg: format!("{}", err),
                            })
                    }
                },
            },
            Err(err) => Err(err),
        }
    }

    fn close_sink(&mut self) {
        drop(self.sink_id);
        self.host_addr.clear();
    }
}

#[derive(Clone)]
pub struct LocalSource {
    pub(crate) recv: Rc<EventReceiver<SinkableMessageImpl>>,
    pub(crate) source_id: SourceId,
    tx: EventSender<SinkableMessageImpl>,
}

impl Source for LocalSource {
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        Rc::get_mut(&mut self.recv).and_then(|recv| futures_executor::block_on(recv.recv()))
    }

    fn source_id(&self) -> SourceId {
        self.source_id
    }

    fn close_source(&mut self) {
        Rc::get_mut(&mut self.recv)
            .iter_mut()
            .for_each(|recv| recv.close());
        futures_executor::block_on(self.tx.closed());
        drop(self.source_id)
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
        Rc<EventReceiver<SinkableMessageImpl>>,
    ),
    Empty(
        SourceId,
        EventSender<SinkableMessageImpl>,
        Rc<EventReceiver<SinkableMessageImpl>>,
    ),
}

impl SourceImpl {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.create_msg_sender(),
            SourceImpl::Kafka(.., terminator_tx, _) => terminator_tx.clone(),
            SourceImpl::Empty(.., terminator_tx, _) => terminator_tx.clone(),
        }
    }

    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.fetch_msg(),
            SourceImpl::Kafka(source, _, terminator_rx) => Rc::get_mut(terminator_rx)
                // should not block thread
                .and_then(|rx| match rx.try_recv() {
                    Ok(message) => Some(message),
                    Err(err) => match err {
                        // if channel has been close, it will terminate all actors
                        tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                            Some(SinkableMessageImpl::LocalMessage(LocalEvent::Terminate {
                                job_id: source.job_id.clone(),
                                to: source.source_id(),
                            }))
                        }
                        _ => None,
                    },
                })
                // if there's no terminate signal, go on
                .or_else(|| source.fetch_msg()),
            SourceImpl::Empty(.., terminator_rx) => match Rc::get_mut(terminator_rx) {
                // block until receive terminate signal
                Some(rx) => futures_executor::block_on(rx.recv()),
                None => None,
            },
        }
    }

    fn close(&mut self) {
        match self {
            SourceImpl::Local(source) => source.close_source(),
            SourceImpl::Kafka(kafka, tx, rx) => {
                kafka.close_source();
                Rc::get_mut(rx).iter_mut().for_each(|recv| recv.close());
                futures_executor::block_on(tx.closed());
            }
            SourceImpl::Empty(id, tx, rx) => {
                drop(id);
                Rc::get_mut(rx).iter_mut().for_each(|recv| recv.close());
                futures_executor::block_on(tx.closed());
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
            SinkImpl::Local(sink) => sink.sink_id(),
            SinkImpl::Remote(sink) => sink.sink_id(),
            SinkImpl::Kafka(kafka) => kafka.sink_id(),
            SinkImpl::Mysql(mysql) => mysql.sink_id(),
            SinkImpl::Empty(sink_id) => *sink_id,
            SinkImpl::Redis(redis) => redis.sink_id(),
        }
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg).await,
            SinkImpl::Remote(sink) => sink.sink(msg).await,
            SinkImpl::Kafka(sink) => sink.sink(msg).await,
            SinkImpl::Mysql(sink) => sink.sink(msg).await,
            SinkImpl::Empty(_) => Ok(DispatchDataEventStatusEnum::Done),
            SinkImpl::Redis(redis) => redis.sink(msg).await,
        }
    }

    fn close_sink(&mut self) {
        match self {
            SinkImpl::Local(sink) => sink.close_sink(),
            SinkImpl::Remote(sink) => sink.close_sink(),
            SinkImpl::Kafka(sink) => sink.close_sink(),
            SinkImpl::Mysql(sink) => sink.close_sink(),
            SinkImpl::Redis(sink) => sink.close_sink(),
            SinkImpl::Empty(id) => drop(id),
        }
    }
}

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
        let datetime = chrono::DateTime::<chrono::Utc>::from(SystemTime::now());

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
                event_time: Some(Timestamp {
                    seconds: datetime.naive_utc().timestamp(),
                    nanos: datetime.naive_utc().timestamp_subsec_nanos() as i32,
                }),
                process_time: None,
                from_operator_id: self.connector_id,
                window: None,
            }));

        result
    }
}

impl Source for Kafka {
    fn fetch_msg(&mut self) -> Option<SinkableMessageImpl> {
        match &self.consumer {
            Some(consumer) => {
                futures_executor::block_on(consumer.fetch(|message| self.process(message)))
            }
            None => None,
        }
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
}

#[async_trait]
impl Sink for Kafka {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match &self.producer {
            Some(producer) => {
                let result = msg.get_kafka_message();
                match result.map_err(|err| err.into()) {
                    Ok(messages) => {
                        for msg in messages {
                            let send_result = producer.send(&msg.key, &msg.payload);
                            if send_result.is_err() {
                                return send_result
                                    .map(|_| DispatchDataEventStatusEnum::Failure)
                                    .map_err(|err| err.into());
                            }
                        }

                        Ok(DispatchDataEventStatusEnum::Done)
                    }
                    Err(err) => Err(err),
                }
            }
            None => Ok(DispatchDataEventStatusEnum::Done),
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
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
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
                        return result.map(|_| DispatchDataEventStatusEnum::Failure);
                    }
                }

                Ok(DispatchDataEventStatusEnum::Done)
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

#[async_trait]
impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException> {
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
                    .map(|_| DispatchDataEventStatusEnum::Done)
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
    use std::rc::Rc;

    use common::event::{LocalEvent, SinkableMessageImpl};
    use proto::common::{
        mysql_desc, operator_info::Details, redis_desc, sink, Entry, Func, KafkaDesc, MysqlDesc,
        RedisDesc, ResourceId,
    };

    use crate::actor::{Sink, SinkImpl, SourceImpl};

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
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let mut kafka_source = SourceImpl::Kafka(
            super::Kafka::with_source_config(&job_id, 0, &desc),
            tx.clone(),
            Rc::new(rx),
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
                assert!(Rc::get_mut(rx)
                    .map(|recv| (*recv).try_recv().is_err())
                    .unwrap_or_default())
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
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let mut source = SourceImpl::Local(super::LocalSource {
                recv: Rc::new(rx),
                source_id: 0,
                tx: tx.clone(),
            });

            source.close();

            assert!(source.fetch_msg().is_none());
            assert!(tx.is_closed());
        }

        {
            let (tx, mut rx) = tokio::sync::mpsc::channel(10);
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
}

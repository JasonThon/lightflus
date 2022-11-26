use crate::dataflow::DataflowTask;
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::new_state_mgt;
use crate::v8_runtime::RuntimeEngine;

use common::collections::lang;
use common::db::MysqlConn;
use common::event::{LocalEvent, SinkableMessage, SinkableMessageImpl};
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer};

use common::redis::RedisClient;
use common::types::{ExecutorId, SinkId, SourceId, TypedValue};
use common::utils;
use prost_types::Timestamp;

use proto::common::{sink, source, DataflowMeta, KafkaDesc, MysqlDesc, OperatorInfo, RedisDesc};
use proto::common::{Entry, KeyedDataEvent};
use proto::common::{HostAddr, ResourceId};
use proto::worker::task_worker_api_client::TaskWorkerApiClient;
use proto::worker::{DispatchDataEventStatusEnum, DispatchDataEventsRequest, StopDataflowRequest};

use std::collections::BTreeMap;

use rayon::prelude::*;
use std::sync::Arc;
use std::time::SystemTime;
use std::vec;

pub type EventReceiver<Input> = crossbeam_channel::Receiver<Input>;
pub type EventSender<Input> = crossbeam_channel::Sender<Input>;

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
            executors.push(ExecutorImpl::Local(LocalExecutor::with_source_and_sink(
                &self.job_id,
                meta.center as ExecutorId,
                source_sink_manager.get_sinks_by_ids(meta.neighbors.clone()),
                source_sink_manager
                    .get_source_by_id(meta.center.clone())
                    .unwrap(),
                self.nodes.get(&meta.center).unwrap().clone(),
            )));

            meta.neighbors.iter().for_each(|id| {
                if !meta_map.contains_key(id) {
                    let neighbors = meta_map
                        .get(id)
                        .map(|slice| (*slice).to_vec())
                        .unwrap_or(vec![]);
                    executors.push(ExecutorImpl::Local(LocalExecutor::with_source_and_sink(
                        &self.job_id,
                        *id as ExecutorId,
                        source_sink_manager.get_sinks_by_ids(neighbors),
                        source_sink_manager.get_source_by_id(*id).unwrap(),
                        self.nodes.get(id).unwrap().clone(),
                    )))
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
    fn run(self) -> tokio::task::JoinHandle<()>;
    fn as_sinkable(&self) -> SinkImpl;
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
}

impl Executor for LocalExecutor {
    fn run(self) -> tokio::task::JoinHandle<()> {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;

        tokio::spawn(async move {
            let isolate = &mut v8::Isolate::new(Default::default());
            let scope = &mut v8::HandleScope::new(isolate);
            let task = DataflowTask::new(self.operator.clone(), new_state_mgt(&self.job_id), scope);
            'outside: loop {
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
                                break 'outside;
                            }
                        },
                    }
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
}

impl ExecutorImpl {
    pub fn run(self) -> tokio::task::JoinHandle<()> {
        match self {
            ExecutorImpl::Local(exec) => exec.run(),
        }
    }

    pub fn as_sinkable(&self) -> SinkImpl {
        match self {
            ExecutorImpl::Local(exec) => exec.as_sinkable(),
        }
    }
}

/*
Source Interface. Each Source should implement it
 */
pub trait Source {
    fn fetch_msg(&self) -> Option<SinkableMessageImpl>;
    fn source_id(&self) -> SourceId;
}

pub struct SourceSinkManger {
    job_id: ResourceId,
    local_source_rx: BTreeMap<SourceId, Arc<EventReceiver<SinkableMessageImpl>>>,
    sources: BTreeMap<SourceId, SourceImpl>,
    local_source_tx: BTreeMap<SourceId, EventSender<SinkableMessageImpl>>,
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
        let (tx, rx) = crossbeam_channel::unbounded();
        let rx_arc = Arc::new(rx);
        self.local_source_rx
            .insert(*executor_id as SourceId, rx_arc.clone());
        self.local_source_tx
            .insert(*executor_id as SourceId, tx.clone());
        let source = LocalSource {
            recv: rx_arc,
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
                    self.sources.insert(
                        *executor_id,
                        SourceImpl::Kafka(Kafka::with_source_config(
                            &self.job_id,
                            *executor_id,
                            conf,
                        )),
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
        self.sources
            .insert(*executor_id, SourceImpl::Empty(*executor_id));
    }

    fn create_empty_sink(&mut self, executor_id: &u32) {
        self.sinks
            .insert(*executor_id, SinkImpl::Empty(*executor_id));
    }
}

impl SourceSinkManger {
    fn new(job_id: &ResourceId) -> SourceSinkManger {
        SourceSinkManger {
            local_source_rx: Default::default(),
            sources: Default::default(),
            local_source_tx: Default::default(),
            sinks: Default::default(),
            job_id: job_id.clone(),
        }
    }
}

#[tonic::async_trait]
pub trait Sink {
    fn sink_id(&self) -> SinkId;
    async fn sink(
        &self,
        msg: SinkableMessageImpl,
    ) -> Result<DispatchDataEventStatusEnum, SinkException>;
}

#[derive(Clone)]
pub struct LocalSink {
    pub(crate) sender: EventSender<SinkableMessageImpl>,
    pub(crate) sink_id: SinkId,
}

#[tonic::async_trait]
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
                .map(|_| DispatchDataEventStatusEnum::Done)
                .map_err(|err| err.into()),
        }
    }
}

#[derive(Clone)]
pub struct RemoteSink {
    pub(crate) sink_id: SinkId,
    pub(crate) host_addr: HostAddr,
}

#[tonic::async_trait]
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
}

#[derive(Clone)]
pub struct LocalSource {
    pub(crate) recv: Arc<EventReceiver<SinkableMessageImpl>>,
    pub(crate) source_id: SourceId,
    tx: EventSender<SinkableMessageImpl>,
}

impl Source for LocalSource {
    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        self.recv.recv().ok()
    }

    fn source_id(&self) -> SourceId {
        self.source_id
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
    Kafka(Kafka),
    Empty(SourceId),
}

impl SourceImpl {
    fn create_msg_sender(&self) -> EventSender<SinkableMessageImpl> {
        let zero_tx = || -> EventSender<SinkableMessageImpl> {
            let (tx, _) = crossbeam_channel::bounded(0);
            tx
        };

        match self {
            SourceImpl::Local(source) => source.create_msg_sender(),
            SourceImpl::Kafka(_) => zero_tx(),
            SourceImpl::Empty(_) => zero_tx(),
        }
    }

    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.fetch_msg(),
            SourceImpl::Kafka(source) => source.fetch_msg(),
            SourceImpl::Empty(_) => None,
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

#[tonic::async_trait]
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
        }))
    }
}

impl Source for Kafka {
    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
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
}

#[tonic::async_trait]
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

#[tonic::async_trait]
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

#[tonic::async_trait]
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
    use proto::common::{mysql_desc, operator_info::Details, sink, Entry};

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
}

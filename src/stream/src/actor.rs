use crate::dataflow::DataflowTask;
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::new_state_mgt;
use crate::v8_runtime::RuntimeEngine;

use common::collections::lang;
use common::event::{LocalEvent, SinkableMessage, SinkableMessageImpl};
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer};

use common::redis::RedisClient;
use common::types::{ExecutorId, SinkId, SourceId, TypedValue};
use common::utils::{self, proto_utils};
use prost_types::Timestamp;

use proto::common::mysql_desc;
use proto::common::{sink, source, DataflowMeta, KafkaDesc, MysqlDesc, OperatorInfo, RedisDesc};
use proto::common::{Entry, KeyedDataEvent};
use proto::common::{HostAddr, ResourceId};
use proto::worker::task_worker_api_client::TaskWorkerApiClient;
use proto::worker::{DispatchDataEventStatusEnum, DispatchDataEventsRequest, StopDataflowRequest};
use sqlx::{Arguments, MySqlPool};

use std::collections::BTreeMap;

use futures_executor;
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
        let external_source = operator_info.filter(|operator| proto_utils::has_source(*operator));
        let external_sink = operator_info.filter(|operator| proto_utils::has_sink(*operator));

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
    pub executor_id: ExecutorId,
    pub(crate) operator: OperatorInfo,

    sinks: Vec<SinkImpl>,
    source: SourceImpl,
}

impl LocalExecutor {
    pub fn with_source_and_sink(
        executor_id: ExecutorId,
        sinks: Vec<SinkImpl>,
        source: SourceImpl,
        operator: OperatorInfo,
    ) -> Self {
        Self {
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
            let task = DataflowTask::new(self.operator.clone(), new_state_mgt(), scope);
            'outside: loop {
                while let Some(msg) = self.source.fetch_msg() {
                    match &msg {
                        LocalMessage(message) => match message {
                            KeyedDataStreamEvent(e) => {
                                if proto_utils::has_source(&self.operator)
                                    || proto_utils::has_sink(&self.operator)
                                {
                                    self.sinks.iter().for_each(|sink| {
                                        let _ = sink.sink(msg.clone());
                                    });
                                } else {
                                    match task.process(e) {
                                        Ok(results) => results.iter().for_each(|event| {
                                            let after_process = KeyedDataStreamEvent(event.clone());
                                            self.sinks.iter().for_each(|sink| {
                                                let _ =
                                                    sink.sink(LocalMessage(after_process.clone()));
                                            });
                                        }),
                                        Err(err) => {
                                            log::error!("process msg failed: {:?}", err);
                                            // TODO fault tolerance
                                        }
                                    }
                                }

                                log::info!("nodeId: {}, msg: {:?}", &self.executor_id, e)
                            }
                            LocalEvent::Terminate { job_id, to } => {
                                log::info!("stopping {:?} at node id {}", job_id, to);
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
                host_addr: proto_utils::get_host_addr(info),
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
        proto_utils::get_source(operator)
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
        proto_utils::get_sink(operator)
            .desc
            .iter()
            .for_each(|desc| match desc {
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
                sink::Desc::Redis(_) => todo!(),
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

pub trait Sink {
    fn sink_id(&self) -> SinkId;
    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException>;
}

#[derive(Clone)]
pub struct LocalSink {
    pub(crate) sender: EventSender<SinkableMessageImpl>,
    pub(crate) sink_id: SinkId,
}

impl Sink for LocalSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
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

impl Sink for RemoteSink {
    fn sink_id(&self) -> SinkId {
        self.sink_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        let ref mut result = futures_executor::block_on(TaskWorkerApiClient::connect(format!(
            "{}:{}",
            &self.host_addr.host, self.host_addr.port
        )));

        result
            .as_mut()
            .map_err(|err| err.into())
            .and_then(|cli| match msg {
                SinkableMessageImpl::LocalMessage(event) => match event {
                    LocalEvent::Terminate { job_id, .. } => {
                        let req = StopDataflowRequest {
                            job_id: Some(job_id),
                        };
                        futures_executor::block_on(cli.stop_dataflow(tonic::Request::new(req)))
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
                        futures_executor::block_on(
                            cli.dispatch_data_events(tonic::Request::new(req)),
                        )
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
            })
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

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg),
            SinkImpl::Remote(sink) => sink.sink(msg),
            SinkImpl::Kafka(sink) => sink.sink(msg),
            SinkImpl::Mysql(sink) => sink.sink(msg),
            SinkImpl::Empty(_) => Ok(DispatchDataEventStatusEnum::Done),
            SinkImpl::Redis(redis) => redis.sink(msg),
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
            &proto_utils::get_kafka_group(config),
            &config.topic,
        ) {
            Ok(consumer) => self_.consumer = Some(Arc::new(consumer)),
            Err(err) => log::error!("kafka source connect failed: {}", err),
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
            &proto_utils::get_kafka_group(config),
            proto_utils::get_kafka_partition(config) as i32,
        ) {
            Ok(producer) => self_.producer = Some(producer),
            Err(err) => log::error!("kafka producer create failed: {}", err),
        }

        self_
    }

    fn process(&self, message: KafkaMessage) -> SinkableMessageImpl {
        let data_type = self.conf.data_type();
        let payload = message.payload.as_slice();
        let val = TypedValue::from_slice_with_type(payload, data_type);
        let mut event = KeyedDataEvent::default();
        let datetime = chrono::DateTime::<chrono::Utc>::from(SystemTime::now());
        let mut event_time = Timestamp::default();

        event_time.seconds = datetime.naive_utc().timestamp();
        event_time.nanos = datetime.naive_utc().timestamp_subsec_nanos() as i32;
        event.event_time = Some(event_time);
        event.job_id = Some(self.job_id.clone());
        event.from_operator_id = self.connector_id;

        let mut data_entry = Entry::default();
        data_entry.set_data_type(data_type);
        data_entry.value = val.get_data();
        event.data = vec![data_entry];

        let mut key_entry = Entry::default();
        let key = TypedValue::from_vec(&message.key);
        key_entry.set_data_type(key.get_type());
        key_entry.value = message.key.clone();
        event.key = Some(key_entry);

        SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(event))
    }
}

impl Source for Kafka {
    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        self.consumer
            .as_ref()
            .and_then(|consumer| consumer.fetch(|message| self.process(message)))
    }

    fn source_id(&self) -> SourceId {
        self.connector_id
    }
}

impl Sink for Kafka {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match &self.producer {
            Some(producer) => {
                let messages_result = msg.get_kafka_message();
                messages_result
                    .map_err(|err| err.into())
                    .and_then(|messages| {
                        for msg in messages {
                            let send_result = producer.send(&msg.key, &msg.payload);
                            if send_result.is_err() {
                                return send_result
                                    .map(|_| DispatchDataEventStatusEnum::Failure)
                                    .map_err(|err| err.into());
                            }
                        }

                        Ok(DispatchDataEventStatusEnum::Done)
                    })
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
    connection_opts: mysql_desc::ConnectionOpts,
}
impl Mysql {
    fn with_config(connector_id: u32, conf: &MysqlDesc) -> Mysql {
        let mut statement = proto_utils::get_mysql_statement(conf);
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

        Mysql {
            connector_id,
            statement,
            extractors,
            connection_opts,
        }
    }

    fn get_arguments(&self, msg: SinkableMessageImpl) -> Vec<Vec<TypedValue>> {
        extract_arguments(self.extractors.as_slice(), msg, "mysql_extractor")
    }

    fn get_uri(&self) -> String {
        let db = &self.connection_opts.database;
        let user = &self.connection_opts.username;
        let password = &self.connection_opts.password;
        let host = &self.connection_opts.host;

        format!("mysql://{user}:{password}@{host}/{db}")
    }

    fn get_statement(&self) -> String {
        self.statement.clone()
    }
}

impl Sink for Mysql {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        let row_arguments = self.get_arguments(msg);
        let pool = futures_executor::block_on(MySqlPool::connect(self.get_uri().as_str()));
        pool.and_then(|conn| {
            for arguments in row_arguments {
                let mut mysql_arg = sqlx::mysql::MySqlArguments::default();
                arguments.iter().for_each(|val| match val {
                    TypedValue::String(v) => mysql_arg.add(v),
                    TypedValue::BigInt(v) => mysql_arg.add(v),
                    TypedValue::Boolean(v) => mysql_arg.add(v),
                    TypedValue::Number(v) => mysql_arg.add(v),
                    _ => {}
                });

                let result = futures_executor::block_on(
                    sqlx::query_with(&self.get_statement(), mysql_arg).execute(&conn),
                );
                if result.is_err() {
                    return result.map(|_| DispatchDataEventStatusEnum::Failure);
                }
            }

            Ok(DispatchDataEventStatusEnum::Done)
        })
        .map_err(|err| err.into())
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

impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
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

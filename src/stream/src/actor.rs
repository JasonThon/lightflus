use crate::dataflow::DataflowTask;
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::new_state_mgt;
use crate::v8_runtime::RuntimeEngine;

use common::collections::lang;
use common::event::{LocalEvent, SinkableMessage, SinkableMessageImpl};
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer};
use common::redis::RedisClient;
use common::types::{ExecutorId, SinkId, SourceId, TypedValue};
use common::utils;
use proto::common::common::{HostAddr, ResourceId};
use proto::common::event::{Entry, KeyedDataEvent};
use proto::common::stream::{
    DataflowMeta, KafkaDesc, MysqlDesc, OperatorInfo, RedisDesc, Sink_oneof_desc, Source_oneof_desc,
};
use proto::worker::cli::{new_dataflow_worker_client, DataflowWorkerConfig};
use proto::worker::worker::{
    DispatchDataEventStatusEnum, DispatchDataEventsRequest, StopDataflowRequest,
};
use protobuf::well_known_types::Timestamp;
use protobuf::RepeatedField;
use sqlx::{Arguments, MySqlPool};

use std::collections::BTreeMap;

use std::sync::Arc;
use std::thread::{spawn, JoinHandle};
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
            meta_map.insert(meta.get_center(), meta.get_neighbors());

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

            meta.get_neighbors().iter().for_each(|id| {
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
    fn run(self) -> JoinHandle<()>;
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
    fn run(self) -> JoinHandle<()> {
        use LocalEvent::KeyedDataStreamEvent;
        use SinkableMessageImpl::LocalMessage;

        spawn(move || {
            let isolate = &mut v8::Isolate::new(Default::default());
            let scope = &mut v8::HandleScope::new(isolate);
            let task = DataflowTask::new(self.operator.clone(), new_state_mgt(), scope);
            'outside: loop {
                while let Some(msg) = self.source.fetch_msg() {
                    match &msg {
                        LocalMessage(message) => match message {
                            KeyedDataStreamEvent(e) => {
                                if self.operator.has_source() || self.operator.has_sink() {
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
    pub fn run(self) -> JoinHandle<()> {
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
                host_addr: info.get_host_addr().clone(),
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
                Source_oneof_desc::kafka(conf) => {
                    self.sources.insert(
                        *executor_id,
                        SourceImpl::Kafka(Kafka::with_source_config(
                            &self.job_id,
                            *executor_id,
                            conf,
                        )),
                    );
                }
                _ => {}
            })
    }

    fn create_external_sink(&mut self, executor_id: &ExecutorId, operator: &OperatorInfo) {
        if self.sinks.contains_key(executor_id) {
            return;
        }
        operator.get_sink().desc.iter().for_each(|desc| match desc {
            Sink_oneof_desc::kafka(kafka) => {
                self.sinks.insert(
                    *executor_id,
                    SinkImpl::Kafka(Kafka::with_sink_config(&self.job_id, *executor_id, kafka)),
                );
            }
            Sink_oneof_desc::mysql(mysql) => {
                self.sinks.insert(
                    *executor_id,
                    SinkImpl::Mysql(Mysql::with_config(*executor_id, mysql)),
                );
            }
            Sink_oneof_desc::redis(_) => todo!(),
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
                .map(|_| DispatchDataEventStatusEnum::DONE)
                .map_err(|err| err.into()),
            _ => Err(SinkException::invalid_message_type()),
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
        let host = if self.host_addr.get_host().is_empty() {
            None
        } else {
            Some(self.host_addr.get_host().to_string())
        };

        let port = if self.host_addr.get_port() != 0 {
            None
        } else {
            Some(self.host_addr.get_port() as u16)
        };

        let cli = new_dataflow_worker_client(DataflowWorkerConfig {
            host,
            port,
            uri: None,
        });

        match msg {
            SinkableMessageImpl::LocalMessage(event) => match event {
                LocalEvent::Terminate { job_id, .. } => {
                    let mut req = StopDataflowRequest::default();
                    req.set_job_id(job_id);
                    cli.stop_dataflow(&req)
                        .map(|_| DispatchDataEventStatusEnum::DISPATCHING)
                        .map_err(|err| SinkException {
                            kind: RemoteSinkFailed,
                            msg: format!("{}", err),
                        })
                }
                LocalEvent::KeyedDataStreamEvent(event) => {
                    let mut req = DispatchDataEventsRequest::default();
                    req.set_events(RepeatedField::from_slice(&[event]));
                    cli.dispatch_data_events(&req)
                        .map(|resp| {
                            for entry in resp.get_statusSet() {
                                match entry.1 {
                                    DispatchDataEventStatusEnum::DISPATCHING => {
                                        return DispatchDataEventStatusEnum::DISPATCHING
                                    }
                                    DispatchDataEventStatusEnum::DONE => continue,
                                    DispatchDataEventStatusEnum::FAILURE => {
                                        return DispatchDataEventStatusEnum::FAILURE
                                    }
                                }
                            }

                            DispatchDataEventStatusEnum::DONE
                        })
                        .map_err(|err| SinkException {
                            kind: RemoteSinkFailed,
                            msg: format!("{}", err),
                        })
                }
            },
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
        }
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg),
            SinkImpl::Remote(sink) => sink.sink(msg),
            SinkImpl::Kafka(sink) => sink.sink(msg),
            SinkImpl::Mysql(sink) => sink.sink(msg),
            SinkImpl::Empty(_) => Ok(DispatchDataEventStatusEnum::DONE),
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
                .get_brokers()
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<String>>()
                .join(",")
                .as_str(),
            config.get_opts().get_group(),
            config.get_topic(),
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
        let producer = run_producer(
            config
                .get_brokers()
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<String>>()
                .join(",")
                .as_str(),
            config.get_topic(),
            config.get_opts(),
        );

        Kafka {
            connector_id: executor_id,
            conf: config.clone(),
            job_id: job_id.clone(),
            consumer: None,
            producer: Some(producer),
        }
    }

    fn process(&self, message: KafkaMessage) -> SinkableMessageImpl {
        let data_type = self.conf.get_data_type();
        let payload = message.payload.as_slice();
        let val = TypedValue::from_slice_with_type(payload, data_type);
        let mut event = KeyedDataEvent::default();
        let datetime = chrono::DateTime::<chrono::Utc>::from(SystemTime::now());
        let mut event_time = Timestamp::default();

        event_time.set_seconds(datetime.naive_utc().timestamp());
        event_time.set_nanos(datetime.naive_utc().timestamp_subsec_nanos() as i32);
        event.set_event_time(event_time);
        event.set_job_id(self.job_id.clone());
        event.set_from_operator_id(self.connector_id);

        let mut data_entry = Entry::default();
        data_entry.set_data_type(data_type);
        data_entry.set_value(val.get_data());
        event.set_data(RepeatedField::from_slice(&[data_entry]));

        let mut key_entry = Entry::default();
        let key = TypedValue::from_vec(&message.key);
        key_entry.set_data_type(key.get_type());
        key_entry.set_value(message.key.clone());
        event.set_key(key_entry);

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
                                    .map(|_| DispatchDataEventStatusEnum::FAILURE)
                                    .map_err(|err| err.into());
                            }
                        }

                        Ok(DispatchDataEventStatusEnum::DONE)
                    })
            }
            None => Ok(DispatchDataEventStatusEnum::DONE),
        }
    }
}

#[derive(Clone)]
pub struct Mysql {
    connector_id: SinkId,
    conf: MysqlDesc,
}
impl Mysql {
    fn with_config(connector_id: u32, conf: &MysqlDesc) -> Mysql {
        Mysql {
            connector_id,
            conf: conf.clone(),
        }
    }

    fn get_arguments(&self, msg: SinkableMessageImpl) -> Vec<Vec<TypedValue>> {
        let mut extractors = self.conf.get_statement().get_extractors().to_vec();

        extractors.sort_by(|v1, v2| v1.get_index().cmp(&v2.get_index()));

        extract_arguments(
            extractors
                .iter()
                .map(|e| e.get_extractor().to_string())
                .collect::<Vec<String>>()
                .as_slice(),
            msg,
            "mysql_extractor",
        )
    }

    fn get_uri(&self) -> String {
        let db = self.conf.get_connection_opts().get_database();
        let user = self.conf.get_connection_opts().get_username();
        let password = self.conf.get_connection_opts().get_password();
        let host = self.conf.get_connection_opts().get_host();

        format!("mysql://{user}:{password}@{host}/{db}")
    }

    fn get_statement(&self) -> String {
        self.conf.get_statement().get_statement().to_string()
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
                    return result.map(|_| DispatchDataEventStatusEnum::FAILURE);
                }
            }

            Ok(DispatchDataEventStatusEnum::DONE)
        })
        .map_err(|err| err.into())
    }
}

#[derive(Clone)]
pub struct Redis {
    connector_id: SinkId,
    conf: RedisDesc,
    client: RedisClient,
}

impl Redis {
    pub fn with_config(connector_id: SinkId, conf: RedisDesc) -> Self {
        let client = RedisClient::new(&conf);
        Self {
            connector_id,
            conf,
            client,
        }
    }
}

impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        let key_extractor = self.conf.get_key_extractor().get_function().to_string();
        let value_extractor = self.conf.get_value_extractor().get_function().to_string();
        let key_values = extract_arguments(
            &[key_extractor, value_extractor],
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
                    .map(|_| DispatchDataEventStatusEnum::DONE)
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
            LocalEvent::KeyedDataStreamEvent(e) => {
                Vec::from_iter(e.get_data().iter().map(|entry| {
                    let val = TypedValue::from_slice(entry.get_value());
                    extractors
                        .iter()
                        .map(|extractor| {
                            let mut rt_engine =
                                RuntimeEngine::new(extractor.as_str(), fn_name, scope);
                            rt_engine.call_one_arg(&val).unwrap_or_default()
                        })
                        .collect::<Vec<TypedValue>>()
                }))
            }
        },
    }
}

mod tests {

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
        use proto::common::common::ResourceId;
        use proto::common::stream::DataflowMeta;
        use proto::common::stream::OperatorInfo;
        use proto::common::stream::{Filter, MysqlDesc, Sink};
        use std::collections::BTreeMap;

        use crate::actor::DataflowContext;

        let mut metas = vec![];
        let mut meta = DataflowMeta::default();
        meta.set_center(0);
        meta.set_neighbors(vec![1]);

        metas.push(meta);
        let mut meta = DataflowMeta::default();
        meta.set_center(1);
        meta.set_neighbors(vec![2]);

        metas.push(meta);

        let mut nodes = BTreeMap::new();
        let mut node_1 = OperatorInfo::default();
        node_1.set_operator_id(0);

        nodes.insert(0, node_1);

        let mut op_1 = OperatorInfo::default();
        op_1.set_operator_id(1);
        op_1.set_filter(Filter::new());
        nodes.insert(1, op_1);

        let mut mysql = OperatorInfo::default();
        mysql.set_operator_id(2);
        let mut sink = Sink::default();
        let desc = MysqlDesc::default();
        sink.set_mysql(desc);
        mysql.set_sink(sink);
        nodes.insert(2, mysql);

        let ctx = DataflowContext::new(ResourceId::default(), metas, nodes);
        let executors = ctx.create_executors();
        assert_eq!(executors.len(), 3);
    }

    #[test]
    pub fn test_get_mysql_arguments() {
        use proto::common::stream::{MysqlDesc, MysqlDesc_Statement};
        use protobuf::RepeatedField;

        use super::Mysql;
        use common::event::LocalEvent;
        use proto::common::event::KeyedDataEvent;

        use crate::actor::SinkableMessageImpl;
        use std::collections::BTreeMap;

        use common::types::TypedValue;
        use proto::common::{event::Entry, stream::MysqlDesc_Statement_Extractor};

        let _setup_guard = setup_v8();

        let mut desc = MysqlDesc::default();
        let mut statement = MysqlDesc_Statement::default();
        statement.set_statement("INSERT INTO table VALUES (?, ?)".to_string());
        statement.set_extractors(RepeatedField::from_iter(
            [
                "function mysql_extractor(a) {return a.v1}",
                "function mysql_extractor(a) {return a.v2}",
            ]
            .iter()
            .map(|extractor| {
                let mut new_extractor = MysqlDesc_Statement_Extractor::default();
                new_extractor.set_extractor(extractor.to_string());
                new_extractor
            }),
        ));
        desc.set_statement(statement);

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
        event.set_data(RepeatedField::from_iter(
            [TypedValue::Object(entry_1)].iter().map(|value| {
                let mut entry = Entry::default();
                entry.set_data_type(value.get_type());
                entry.set_value(value.get_data());

                entry
            }),
        ));
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

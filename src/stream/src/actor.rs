use crate::dataflow::DataflowTask;
use crate::err::{ErrorKind::RemoteSinkFailed, SinkException};
use crate::state::new_state_mgt;
use crate::v8_runtime::RuntimeEngine;

use common::collections::lang;
use common::event::LocalEvent;
use common::kafka::{run_consumer, run_producer, KafkaConsumer, KafkaProducer};
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
use sqlx::{Arguments, Execute, MySqlPool};

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

    pub fn create_executors(&self) -> Vec<ExecutorImpl> {
        let ref mut source_sink_manager = SourceSinkManger::new(&self.job_id);
        let mut meta_map = BTreeMap::new();
        self.meta.iter().for_each(|meta| {
            meta_map.insert(meta.get_center(), meta.get_neighbors());
            source_sink_manager.create_local(&(meta.center as ExecutorId));

            meta.neighbors.iter().for_each(|node_id| {
                let executor_id = *node_id as ExecutorId;
                let operator_info = self.nodes.get(&executor_id);
                let remote_node =
                    operator_info.filter(|operator| utils::is_remote_operator(*operator));
                let external_source = operator_info.filter(|operator| (*operator).has_source());
                let external_sink = operator_info.filter(|operator| (*operator).has_sink());

                if remote_node.is_some() {
                    remote_node.iter().for_each(|operator| {
                        source_sink_manager.create_remote_sink(&executor_id, *operator)
                    })
                } else if external_source.is_some() {
                    external_source.iter().for_each(|operator| {
                        source_sink_manager.create_external_source(&executor_id, *operator)
                    })
                } else if external_sink.is_some() {
                    external_sink.iter().for_each(|operator| {
                        source_sink_manager.create_external_sink(&executor_id, *operator)
                    })
                } else {
                    source_sink_manager.create_local(&executor_id)
                }
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

pub trait SinkableMessage {
    fn get_key(&self) -> Vec<u8>;
    fn get_payload(&self) -> Vec<u8>;
}

#[derive(Clone, Debug)]
pub enum SinkableMessageImpl {
    LocalMessage(LocalEvent),
}

impl SinkableMessage for SinkableMessageImpl {
    fn get_key(&self) -> Vec<u8> {
        match self {
            SinkableMessageImpl::LocalMessage(event) => match event {
                LocalEvent::Terminate { .. } => vec![],
                LocalEvent::KeyedDataStreamEvent(e) => e.get_key().get_value().to_vec(),
            },
        }
    }

    fn get_payload(&self) -> Vec<u8> {
        match self {
            SinkableMessageImpl::LocalMessage(event) => match event {
                LocalEvent::Terminate { .. } => vec![],
                LocalEvent::KeyedDataStreamEvent(e) => protobuf::Message::write_to_bytes(e)
                    .map_err(|err| log::error!("protobuf marshal failed: {}", err))
                    .unwrap_or(vec![]),
            },
        }
    }
}

unsafe impl Send for SinkableMessageImpl {}
unsafe impl Sync for SinkableMessageImpl {}

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
        }
    }

    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        match self {
            SourceImpl::Local(source) => source.fetch_msg(),
            SourceImpl::Kafka(source) => source.fetch_msg(),
        }
    }
}

#[derive(Clone)]
pub enum SinkImpl {
    Local(LocalSink),
    Remote(RemoteSink),
    Kafka(Kafka),
    Mysql(Mysql),
}

impl Sink for SinkImpl {
    fn sink_id(&self) -> SinkId {
        match self {
            SinkImpl::Local(sink) => sink.sink_id(),
            SinkImpl::Remote(sink) => sink.sink_id(),
            SinkImpl::Kafka(kafka) => kafka.sink_id(),
            SinkImpl::Mysql(mysql) => mysql.sink_id(),
        }
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        match self {
            SinkImpl::Local(sink) => sink.sink(msg),
            SinkImpl::Remote(sink) => sink.sink(msg),
            SinkImpl::Kafka(sink) => sink.sink(msg),
            SinkImpl::Mysql(sink) => sink.sink(msg),
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
}

impl Source for Kafka {
    fn fetch_msg(&self) -> Option<SinkableMessageImpl> {
        self.consumer.as_ref().and_then(|consumer| {
            let data_type = self.conf.get_data_type();
            consumer.fetch(|message| {
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

                let mut entry = Entry::default();
                entry.set_data_type(data_type);
                entry.set_value(val.get_data());
                event.set_data(RepeatedField::from_slice(&[entry]));

                SinkableMessageImpl::LocalMessage(LocalEvent::KeyedDataStreamEvent(event))
            })
        })
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
            Some(producer) => producer
                .send(msg.get_key().as_slice(), msg.get_payload().as_slice())
                .map(|_| DispatchDataEventStatusEnum::DONE)
                .map_err(|err| err.into()),
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

    fn get_actual_statements(&self, msg: SinkableMessageImpl) -> Vec<String> {
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let mut extractors = self.conf.get_statement().get_extractors().to_vec();

        extractors.sort_by(|v1, v2| v1.get_index().cmp(&v2.get_index()));

        let row_arguments = match msg {
            SinkableMessageImpl::LocalMessage(event) => match event {
                LocalEvent::Terminate { .. } => vec![],
                LocalEvent::KeyedDataStreamEvent(e) => {
                    Vec::from_iter(e.get_data().iter().map(|entry| {
                        let val = TypedValue::from_slice(entry.get_value());
                        extractors
                            .iter()
                            .map(|extractor| {
                                let mut rt_engine = RuntimeEngine::new(
                                    extractor.get_extractor(),
                                    "mysql_extractor",
                                    scope,
                                );
                                rt_engine.call_one_arg(&val).unwrap_or_default()
                            })
                            .collect::<Vec<TypedValue>>()
                    }))
                }
            },
        };

        row_arguments
            .iter()
            .map(|values| {
                let mut arguments = sqlx::mysql::MySqlArguments::default();

                values.iter().for_each(|val| match val {
                    TypedValue::String(v) => arguments.add(v),
                    TypedValue::BigInt(v) => arguments.add(v),
                    TypedValue::Boolean(v) => arguments.add(v),
                    TypedValue::Number(v) => arguments.add(v),
                    _ => arguments.add(""),
                });
                sqlx::query_with::<sqlx::MySql, sqlx::mysql::MySqlArguments>(
                    self.conf.get_statement().get_statement(),
                    arguments,
                )
                .sql()
                .to_string()
            })
            .collect()
    }

    fn get_uri(&self) -> String {
        let db = self.conf.get_connection_opts().get_database();
        let user = self.conf.get_connection_opts().get_username();
        let password = self.conf.get_connection_opts().get_password();
        let host = self.conf.get_connection_opts().get_host();

        format!("mysql://{user}:{password}@{host}/{db}")
    }
}

impl Sink for Mysql {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        let statements: Vec<String> = self.get_actual_statements(msg);
        let pool = futures_executor::block_on(MySqlPool::connect(self.get_uri().as_str()));
        pool.and_then(|conn| {
            for statement in statements {
                let result = futures_executor::block_on(sqlx::query(&statement).execute(&conn));
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
}

impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    fn sink(&self, msg: SinkableMessageImpl) -> Result<DispatchDataEventStatusEnum, SinkException> {
        todo!()
    }
}

mod tests {

    #[test]
    fn test_new_dataflow_context() {
        use proto::common::common::ResourceId;
        use proto::common::stream::DataflowMeta;
        use proto::common::stream::OperatorInfo;
        use proto::common::stream::{KafkaDesc, Source};
        use protobuf::RepeatedField;
        use std::collections::BTreeMap;

        use crate::actor::DataflowContext;

        let mut metas = vec![];
        let mut meta = DataflowMeta::default();
        meta.set_center(0);
        metas.push(meta);

        let mut nodes = BTreeMap::new();
        let mut kafka = OperatorInfo::default();
        let mut source = Source::default();
        let mut desc = KafkaDesc::new();

        desc.set_brokers(RepeatedField::from_slice(&["localhost:9001".to_string()]));
        desc.set_topic("test".to_string());
        source.set_kafka(desc);
        kafka.set_source(source);
        nodes.insert(0, kafka);

        let ctx = DataflowContext::new(ResourceId::default(), metas, nodes);
        let executors = ctx.create_executors();
        assert_eq!(executors.len(), 1);
    }
}

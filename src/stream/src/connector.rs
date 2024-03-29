use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
    task::Poll,
};

use common::{
    db::MysqlConn,
    event::{LocalEvent, StreamEvent},
    kafka::{run_consumer, run_producer, KafkaConsumer, KafkaMessage, KafkaProducer},
    redis::RedisClient,
    types::{ExecutorId, SinkId, SourceId, TypedValue},
    utils::times::{now, now_timestamp},
};
use prost::Message;

use proto::common::{
    operator_info::{self, Details},
    sink, source, Entry, KafkaDesc, KeyedDataEvent, KeyedEventSet, MysqlDesc, OperatorInfo,
    RedisDesc, ResourceId,
};

use tokio::sync::mpsc::error::TryRecvError;
use tonic::async_trait;

use crate::{
    err::{BatchSinkException, SinkException},
    new_event_channel,
    v8_runtime::RuntimeEngine,
    Receiver, Sender,
};
/**
 * Source and Sink Connectors
 *
 * A Connector represents an external Data Source (e.g. Kafka, MQ) or Data Sink (e.g. Kafka, Database).
 * In e2e argument, Lightflus have to guarentee that each event generated by Data Source that will be processed by a DAG pipeline,
 * should be sent to Data Sink with three consistency level:
 *
 * - No Guarentee: Lightflus does not make sure each event will be end-to-end consistent.
 * - At-Least-Once: Lightflus makes sure each event will be processed end-to-end at least once.
 * - Exactly-Once: Lightflus makes sure each event will be processed end-to-end exactly-once.
 *
 * To guarentee the last two levels, Lightflus must has the ability of fault tolerance.
 * But users have to make a trade-off between fault tolerance, latency and throughput:
 *
 * - If developer configure Lightflus in NO-GUARANTEE, latency and throughput will be in the highest performance, however, message can be lost.
 * - If developer configure Lightflus in AT-LEAST-ONCE, latency and throughput performance will be lower and message may be sent to sink multiple times.
 * - If developer configure Lightflus in EXACYLY-ONCE, latency and throughput performance is in the lowest (you may not accept it) but message can be sure delivered only once.
 */

/// The trait for a data source.
#[async_trait]
pub trait Source {
    /// source id of the Source
    fn source_id(&self) -> SourceId;

    /// gracefully close this source
    async fn close_source(&mut self);

    /// fetch next message asynchronously
    /// no matter what form the source message is, they will all be turned into [`LocalEvent`]
    async fn next(&mut self) -> Option<LocalEvent>;

    fn poll_next(&mut self, cx: &mut std::task::Context<'_>)
        -> std::task::Poll<Option<LocalEvent>>;
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
    async fn sink(&mut self, msg: LocalEvent) -> Result<(), SinkException>;

    async fn batch_sink(&mut self, event_set: KeyedEventSet) -> Result<(), BatchSinkException>;

    /**
     * Gracefully close sink
     */
    fn close_sink(&mut self);
}

pub enum SourceImpl {
    Kafka(Kafka, Sender<LocalEvent>, Receiver<LocalEvent>),
    Empty(SourceId, Sender<LocalEvent>, Receiver<LocalEvent>),
}

#[async_trait]
impl Source for SourceImpl {
    fn source_id(&self) -> SourceId {
        match self {
            SourceImpl::Kafka(source, _, _) => source.source_id(),
            SourceImpl::Empty(source_id, _, _) => *source_id,
        }
    }

    async fn next(&mut self) -> Option<LocalEvent> {
        match self {
            Self::Kafka(source, _, terminator_rx) => {
                match terminator_rx.try_recv() {
                    Ok(message) => Some(message),
                    Err(err) => match err {
                        // if channel has been close, it will terminate all actors
                        TryRecvError::Disconnected => Some(LocalEvent::Terminate {
                            job_id: source.job_id.clone(),
                            to: source.source_id(),
                            event_time: now().timestamp_millis(),
                        }),
                        _ => source.next().await,
                    },
                }
            }
            Self::Empty(.., terminator_rx) => terminator_rx.recv().await,
        }
    }

    fn poll_next(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<LocalEvent>> {
        match self {
            Self::Kafka(source, _, _) => source.poll_next(cx),
            Self::Empty(.., terminator_rx) => terminator_rx.poll_recv(cx),
        }
    }

    async fn close_source(&mut self) {
        match self {
            Self::Kafka(kafka, tx, rx) => {
                rx.close();
                tokio::join!(kafka.close_source(), tx.closed());
            }
            Self::Empty(id, tx, rx) => {
                drop(id);
                rx.close();
                tx.closed().await;
            }
        }
    }
}

impl From<(&ResourceId, SourceId, &operator_info::Details)> for SourceImpl {
    fn from(args: (&ResourceId, SourceId, &operator_info::Details)) -> Self {
        let (tx, rx) = new_event_channel(1);
        match args.2 {
            operator_info::Details::Source(source) => match source.desc.as_ref() {
                Some(desc) => match desc {
                    source::Desc::Kafka(conf) => {
                        SourceImpl::Kafka(Kafka::with_source_config(args.0, args.1, conf), tx, rx)
                    }
                },
                None => SourceImpl::Empty(args.1, tx, rx),
            },
            _ => SourceImpl::Empty(args.1, tx, rx),
        }
    }
}

pub enum SinkImpl {
    Kafka(Kafka),
    Mysql(Mysql),
    Redis(Redis),
    Empty(SinkId),
}

#[async_trait]
impl Sink for SinkImpl {
    fn sink_id(&self) -> SinkId {
        match self {
            Self::Kafka(kafka) => kafka.sink_id(),
            Self::Mysql(mysql) => mysql.sink_id(),
            Self::Empty(sink_id) => *sink_id,
            Self::Redis(redis) => redis.sink_id(),
        }
    }

    async fn sink(&mut self, msg: LocalEvent) -> Result<(), SinkException> {
        match self {
            Self::Kafka(sink) => sink.sink(msg).await,
            Self::Mysql(sink) => sink.sink(msg).await,
            Self::Empty(_) => Ok(()),
            Self::Redis(redis) => redis.sink(msg).await,
        }
    }

    fn close_sink(&mut self) {
        match self {
            Self::Kafka(sink) => sink.close_sink(),
            Self::Mysql(sink) => sink.close_sink(),
            Self::Redis(sink) => sink.close_sink(),
            Self::Empty(id) => drop(id),
        }
    }

    async fn batch_sink(&mut self, event_set: KeyedEventSet) -> Result<(), BatchSinkException> {
        match self {
            Self::Kafka(sink) => sink.batch_sink(event_set).await,
            Self::Mysql(sink) => sink.batch_sink(event_set).await,
            Self::Empty(_) => Ok(()),
            Self::Redis(redis) => redis.batch_sink(event_set).await,
        }
    }
}

impl From<(&ResourceId, &OperatorInfo)> for SinkImpl {
    fn from((resource_id, info): (&ResourceId, &OperatorInfo)) -> Self {
        match &info.details {
            Some(detail) => match detail {
                Details::Sink(sink) => match &sink.desc {
                    Some(desc) => match desc {
                        sink::Desc::Kafka(desc) => SinkImpl::Kafka(Kafka::with_sink_config(
                            resource_id,
                            info.operator_id,
                            desc,
                        )),
                        sink::Desc::Mysql(desc) => {
                            SinkImpl::Mysql(Mysql::with_config(info.operator_id, desc))
                        }
                        sink::Desc::Redis(desc) => {
                            SinkImpl::Redis(Redis::with_config(info.operator_id, desc))
                        }
                    },
                    None => Self::Empty(info.operator_id),
                },
                _ => todo!(),
            },
            None => Self::Empty(info.operator_id),
        }
    }
}

/// An unified implementation for Kafka Source and Sink.
pub struct Kafka {
    connector_id: SourceId,
    conf: KafkaDesc,
    job_id: ResourceId,
    consumer: Option<KafkaConsumer>,
    producer: Option<KafkaProducer>,
    job_id_hash: u64,
}

impl Kafka {
    pub fn with_source_config(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        config: &KafkaDesc,
    ) -> Kafka {
        let ref mut hasher = DefaultHasher::new();
        Hash::hash(job_id, hasher);
        let job_id_hash = hasher.finish();

        let mut this = Kafka {
            connector_id: executor_id,
            conf: config.clone(),
            job_id: job_id.clone(),
            consumer: None,
            producer: None,
            job_id_hash,
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
            Ok(consumer) => this.consumer = Some(consumer),
            Err(err) => tracing::error!("kafka source connect failed: {}", err),
        };

        this
    }

    pub fn with_sink_config(
        job_id: &ResourceId,
        executor_id: ExecutorId,
        config: &KafkaDesc,
    ) -> Kafka {
        let ref mut hasher = DefaultHasher::new();
        Hash::hash(job_id, hasher);
        let job_id_hash = hasher.finish();

        let mut this = Kafka {
            connector_id: executor_id,
            conf: config.clone(),
            job_id: job_id.clone(),
            consumer: None,
            producer: None,
            job_id_hash,
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
            Ok(producer) => this.producer = Some(producer),
            Err(err) => tracing::error!("kafka producer create failed: {}", err),
        }

        this
    }

    fn process(&self, message: KafkaMessage) -> LocalEvent {
        let data_type = self.conf.data_type();
        let key = TypedValue::from_slice(&message.key);
        let val = TypedValue::from_slice_with_type(&message.payload, data_type);
        let event_id = self.generate_new_event_id();

        let result = LocalEvent::KeyedDataStreamEvent(KeyedDataEvent {
            job_id: Some(self.job_id.clone()),
            key: Some(Entry {
                data_type: key.get_type() as i32,
                value: key.get_data_bytes(),
            }),
            to_operator_id: 0,
            data: vec![Entry {
                data_type: self.conf.data_type() as i32,
                value: val.get_data_bytes(),
            }],

            event_time: message.timestamp.unwrap_or_else(|| now_timestamp()),
            from_operator_id: self.connector_id,
            window: None,
            event_id,
        });

        result
    }

    fn generate_new_event_id(&self) -> i64 {
        const EPOCH: i64 = 1640966400;

        let timestamp = now_timestamp();
        let diff = timestamp - EPOCH;
        diff + (self.job_id_hash as i64)
    }
}

#[async_trait]
impl Source for Kafka {
    fn source_id(&self) -> SourceId {
        self.connector_id
    }

    async fn close_source(&mut self) {
        self.conf.clear();
        self.job_id.clear();
        drop(self.connector_id);
        self.consumer.iter().for_each(|consumer| {
            consumer.unsubscribe();
            drop(consumer)
        })
    }

    async fn next(&mut self) -> Option<LocalEvent> {
        match &self.consumer {
            Some(consumer) => consumer.fetch(|message| self.process(message)).await,
            None => None,
        }
    }

    fn poll_next(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Option<LocalEvent>> {
        Poll::Ready(
            self.consumer
                .as_ref()
                .and_then(|consumer| consumer.blocking_fetch(|message| self.process(message))),
        )
    }
}

#[async_trait]
impl Sink for Kafka {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(&mut self, msg: LocalEvent) -> Result<(), SinkException> {
        match &self.producer {
            Some(producer) => {
                let result = msg.to_kafka_message();
                match result.map_err(|err| err.into()) {
                    Ok(messages) => {
                        for msg in messages {
                            let send_result = producer.send(&msg.key, &msg.payload).await;
                            if send_result.is_err() {
                                return send_result.map_err(|err| err.into());
                            }
                        }

                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            None => Ok(()),
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

    async fn batch_sink(&mut self, event_set: KeyedEventSet) -> Result<(), BatchSinkException> {
        match &self.producer {
            Some(producer) => {
                for event in event_set
                    .events
                    .into_iter()
                    .map(|event| LocalEvent::KeyedDataStreamEvent(event))
                {
                    let kafka_msg = event.to_kafka_message();
                    match kafka_msg {
                        Ok(messages) => {
                            for msg in messages {
                                match producer.send(&msg.key, &msg.payload).await {
                                    Err(err) => {
                                        tracing::error!(
                                            "sink [{:?}] to kafka failed: {}",
                                            &msg,
                                            err
                                        )
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(err) => {
                            tracing::error!(
                                "LocalEvent {:?} to KafkaMessage failed: {:?}",
                                &event,
                                err
                            )
                        }
                    }
                }

                Ok(())
            }
            None => Ok(()),
        }
    }
}

/// An unified implementation for Mysql Source and Sink
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

    fn get_arguments(&self, msg: &LocalEvent) -> Vec<Vec<TypedValue>> {
        extract_arguments(self.extractors.as_slice(), msg, "mysql_extractor")
    }
}

#[async_trait]
impl Sink for Mysql {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(&mut self, msg: LocalEvent) -> Result<(), SinkException> {
        let row_arguments = self.get_arguments(&msg);
        for arguments in row_arguments {
            let result = self
                .conn
                .execute(&self.statement, arguments)
                .await
                .map_err(|err| err.into());
            if result.is_err() {
                return result.map(|_| {});
            }
        }

        Ok(())
    }

    fn close_sink(&mut self) {
        self.conn.close();
        self.extractors.clear();
        drop(self.connector_id);
        self.statement.clear();
    }

    async fn batch_sink(&mut self, event_set: KeyedEventSet) -> Result<(), BatchSinkException> {
        let row_arguments = event_set
            .events
            .into_iter()
            .map(|event| LocalEvent::KeyedDataStreamEvent(event))
            .map(|event| self.get_arguments(&event))
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        self.conn
            .execute(&self.statement, row_arguments)
            .await
            .map(|_| {})
            .map_err(|err| {
                tracing::error!("execute mysql statement failed: {}", err);
                BatchSinkException::from(err)
            })?;

        Ok(())
    }
}

/// An unified implement for Redis Source and Sink
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
const REDIS_EXTRACTOR_FUN_NAME: &str = "redis_extractor";
#[async_trait]
impl Sink for Redis {
    fn sink_id(&self) -> SinkId {
        self.connector_id
    }

    async fn sink(&mut self, msg: LocalEvent) -> Result<(), SinkException> {
        let key_values = extract_arguments(
            &[self.key_extractor.clone(), self.value_extractor.clone()],
            &msg,
            REDIS_EXTRACTOR_FUN_NAME,
        );
        let kvs = Vec::from_iter(key_values.iter().map(|kv| (&kv[0], &kv[1])));
        self.client
            .set_multiple(kvs.as_slice())
            .map_err(|err| err.into())
    }

    fn close_sink(&mut self) {
        drop(self.connector_id);
        self.key_extractor.clear();
        self.value_extractor.clear();
    }

    async fn batch_sink(&mut self, mut event_set: KeyedEventSet) -> Result<(), BatchSinkException> {
        let mut kv_set = BTreeMap::new();
        event_set.events.sort_by_key(|event| event.event_time);
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);

        event_set.events.into_iter().for_each(|event| {
            extract_arguments_scope(
                &[self.key_extractor.clone(), self.value_extractor.clone()],
                &LocalEvent::KeyedDataStreamEvent(event),
                REDIS_EXTRACTOR_FUN_NAME,
                scope,
            )
            .into_iter()
            .filter(|kv| !kv.is_empty())
            .map(|kv| (kv[0].clone(), kv[1].clone()))
            .for_each(|(key, value)| {
                kv_set.insert(key, value);
            })
        });
        self.client
            .set_multiple(kv_set.iter().collect::<Vec<_>>().as_slice())
            .map_err(|err| err.into())
    }
}

fn extract_arguments_scope(
    extractors: &[String],
    event: &LocalEvent,
    fn_name: &str,
    scope: &mut v8::HandleScope<'_, ()>,
) -> Vec<Vec<TypedValue>> {
    match event {
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
    }
}

fn extract_arguments(
    extractors: &[String],
    event: &LocalEvent,
    fn_name: &str,
) -> Vec<Vec<TypedValue>> {
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    match event {
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
    }
}

#[cfg(test)]
mod tests {
    use proto::common::{
        mysql_desc, redis_desc, Entry, Func, KafkaDesc, MysqlDesc, RedisDesc, ResourceId,
    };

    use crate::{new_event_channel, MOD_TEST_START};

    use super::{Sink, SinkImpl, Source, SourceImpl};

    struct SetupGuard {}

    impl Drop for SetupGuard {
        fn drop(&mut self) {}
    }

    fn setup_v8() -> SetupGuard {
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
    pub fn test_get_mysql_arguments() {
        use proto::common::MysqlDesc;

        use super::Mysql;
        use common::event::LocalEvent;
        use proto::common::KeyedDataEvent;

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
            value: value.get_data_bytes(),
        }));
        let message = LocalEvent::KeyedDataStreamEvent(event);
        let arguments = mysql.get_arguments(&message);
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
            rx,
        );

        let mut kafka_sink = SinkImpl::Kafka(super::Kafka::with_sink_config(&job_id, 0, &desc));

        kafka_source.close_source().await;

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
                let result = rx.try_recv();
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
}

use chrono::Duration;

use crate::common::{
    mysql_desc::{self, Statement},
    operator_info::Details,
    sink, source,
    trigger::Watermark,
    window::{self, FixedWindow, SessionWindow, SlidingWindow},
    Ack, DataTypeEnum, Dataflow, Entry, SubDataflowId, Func, Heartbeat, HostAddr, KafkaDesc,
    KeyedDataEvent, MysqlDesc, OperatorInfo, RedisDesc, ResourceId, Response, Sink, Source, Time,
    Trigger, Window,
};

pub const SUCCESS_RPC_RESPONSE: &str = "success";

const RESOURCE_ID_SCHEMA: &str = r#"{
    "name": "ResourceId", 
    "type": "record", 
    "fields": [{
        "name": "resource_id", 
        "type": "string"
    },
    {
        "name": "namespace_id", 
        "type": "string"
    }]
}"#;

const ENTRY_SCHEMA: &str = r#"{
    "name": "Entry",
    "type": "record", 
    "fields": [
        {
            "name": "data_type", 
            "type": "int"
        },
        {
            "name": "value", 
            "type": "bytes"
        }
    ]
}"#;

const WINDOW_SCHEMA: &str = r#"{
    "name": "Window",
    "type": "record",
    "fields": [
        {
            "name": "start_time",
            "type": "long"
        },
        {
            "name": "end_time",
            "type": "long"
        }
    ]
}"#;

const KEYED_DATA_EVENT_SCHEMA: &str = r#"{
    "type": "record",
    "name": "KeyedDataEvent",
    "fields": [
        {
            "name": "job_id", 
            "type": "ResourceId"
        },
        {
            "name": "key", 
            "type": "Entry", 
        },
        {
            "name": "data", 
            "type": "array", 
            "items": "Entry"
        }
        {
            "name": "to_operator_id", 
            "type": "int"
        }
        {
            "name": "event_time", 
            "type": "long"
        }
        {
            "name": "from_operator_id", 
            "type": "int"
        }
        {
            "name": "window", 
            "type": "Window"
        }
        {
            "name": "event_id", 
            "type": "long"
        }
    ]
}"#;

impl OperatorInfo {
    pub fn has_source(&self) -> bool {
        self.details
            .as_ref()
            .filter(|details| match details {
                Details::Source(_) => true,
                _ => false,
            })
            .is_some()
    }

    pub fn has_sink(&self) -> bool {
        self.details
            .as_ref()
            .filter(|details| match details {
                Details::Sink(_) => true,
                _ => false,
            })
            .is_some()
    }

    pub fn get_host_addr(&self) -> HostAddr {
        self.host_addr
            .as_ref()
            .map(|addr| addr.clone())
            .unwrap_or_default()
    }

    pub fn get_host_addr_ref(&self) -> Option<&HostAddr> {
        self.host_addr.as_ref()
    }

    pub fn get_source(&self) -> Source {
        self.details
            .as_ref()
            .and_then(|details| match details {
                Details::Source(source) => Some(source.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn get_sink(&self) -> Sink {
        self.details
            .as_ref()
            .and_then(|details| match details {
                Details::Sink(sink) => Some(sink.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn has_window(&self) -> bool {
        self.details
            .as_ref()
            .map(|details| match details {
                Details::Window(_) => true,
                _ => false,
            })
            .unwrap_or_default()
    }

    pub fn get_window(&self) -> Window {
        self.details
            .as_ref()
            .and_then(|details| match details {
                Details::Window(window) => Some(window.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }
}

impl Window {
    pub fn get_value(&self) -> Option<&window::Value> {
        self.value.as_ref()
    }

    pub fn get_trigger(&self) -> Option<&Trigger> {
        self.trigger.as_ref()
    }
}

impl KafkaDesc {
    pub(crate) fn check(&self) -> Result<(), DataflowValidateError> {
        if self.brokers.is_empty() {
            Err(DataflowValidateError::MissingKafkaBrokers)
        } else if self.data_type() == DataTypeEnum::Unspecified {
            Err(DataflowValidateError::MissingKafkaDataType)
        } else if self.topic.is_empty() {
            Err(DataflowValidateError::MissingKafkaTopic)
        } else {
            Ok(())
        }
    }

    pub fn get_kafka_group(&self) -> String {
        self.opts
            .as_ref()
            .and_then(|opts| opts.group.clone())
            .unwrap_or_default()
    }

    pub fn get_kafka_partition(&self) -> u32 {
        self.opts
            .as_ref()
            .and_then(|opts| opts.partition)
            .unwrap_or_default()
    }
}

impl MysqlDesc {
    pub fn get_mysql_statement(&self) -> Statement {
        self.statement
            .as_ref()
            .map(|statement| statement.clone())
            .unwrap_or_default()
    }

    pub(crate) fn check(&self) -> Result<(), DataflowValidateError> {
        if self.connection_opts.is_none() {
            Err(DataflowValidateError::MissingMysqlConnectionOpts)
        } else if self.statement.is_none() {
            Err(DataflowValidateError::MissingMysqlStatement)
        } else if self
            .statement
            .as_ref()
            .filter(|statement| statement.statement.is_empty())
            .is_some()
        {
            Err(DataflowValidateError::MissingMysqlStatement)
        } else {
            Ok(())
        }
    }
}

impl RedisDesc {
    pub(crate) fn check(&self) -> Result<(), DataflowValidateError> {
        if self.key_extractor.is_none() {
            Err(DataflowValidateError::MissingRedisKeyExtractor)
        } else if self.value_extractor.is_none() {
            Err(DataflowValidateError::MissingRedisValueExtractor)
        } else if self.connection_opts.is_none() {
            Err(DataflowValidateError::MissingRedisConnectionOpts)
        } else {
            match self.connection_opts.as_ref() {
                Some(opts) => {
                    if opts.host.is_empty() {
                        Err(DataflowValidateError::MissingRedisHost)
                    } else if opts.tls {
                        if opts.username.is_empty() || opts.password.is_empty() {
                            Err(DataflowValidateError::MissingRedisTlsConfig)
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
                None => Err(DataflowValidateError::MissingRedisConnectionOpts),
            }
        }
    }
}

impl Dataflow {
    pub fn validate(&self) -> Result<(), DataflowValidateError> {
        if self.job_id.is_none() {
            return Err(DataflowValidateError::MissingResourceId);
        }
        let mut metas = self.meta.to_vec();
        metas.sort_by(|prev, next| prev.center.cmp(&next.center));

        for meta in &metas {
            let result = self.check_operator(meta.center);
            if result.is_err() {
                return result;
            }

            for neighbor in &meta.neighbors {
                if neighbor < &meta.center {
                    return Err(DataflowValidateError::CyclicDataflow);
                }

                let result = self.check_operator(*neighbor);
                if result.is_err() {
                    return result;
                }
            }
        }

        return Ok(());
    }

    pub fn check_operator(&self, node_id: u32) -> Result<(), DataflowValidateError> {
        if !self.nodes.contains_key(&node_id) {
            Err(DataflowValidateError::OperatorInfoMissing(format!(
                "operatorInfo of node {} is missing",
                node_id
            )))
        } else {
            let operator = self.nodes.get(&node_id).unwrap();

            match operator.details.as_ref() {
                Some(detail) => match detail {
                    Details::Source(source) => source.check(),
                    Details::Sink(sink) => sink.check(),
                    _ => Ok(()),
                },
                None => return Err(DataflowValidateError::OperatorDetailMissing(node_id)),
            }
        }
    }

    pub fn get_job_id(&self) -> ResourceId {
        self.job_id
            .as_ref()
            .map(|id| id.clone())
            .unwrap_or_default()
    }

    pub fn get_execution_id_ref(&self) -> Option<&SubDataflowId> {
        self.execution_id.as_ref()
    }
}

#[derive(Debug, serde::Serialize)]
pub enum DataflowValidateError {
    MissingRedisConnectionOpts,
    MissingRedisStatement,
    MissingResourceId,
    OperatorInfoMissing(String),
    CyclicDataflow,
    OperatorDetailMissing(u32),
    MissingSourceDesc,
    MissingSinkDesc,
    MissingRedisKeyExtractor,
    MissingRedisValueExtractor,
    MissingMysqlConnectionOpts,
    MissingMysqlStatement,
    MissingRedisHost,
    MissingRedisTlsConfig,
    MissingKafkaBrokers,
    MissingKafkaDataType,
    MissingKafkaTopic,
}

impl Source {
    pub(crate) fn check(&self) -> Result<(), DataflowValidateError> {
        match self.desc.as_ref() {
            Some(desc) => match desc {
                source::Desc::Kafka(kafka) => kafka.check(),
            },
            None => Err(DataflowValidateError::MissingSourceDesc),
        }
    }
}

impl Sink {
    pub(crate) fn check(&self) -> Result<(), DataflowValidateError> {
        match self.desc.as_ref() {
            Some(desc) => match desc {
                sink::Desc::Redis(redis) => redis.check(),
                sink::Desc::Kafka(kafka) => kafka.check(),
                sink::Desc::Mysql(mysql) => mysql.check(),
            },
            None => Err(DataflowValidateError::MissingSinkDesc),
        }
    }
}

impl KeyedDataEvent {
    #[inline]
    pub fn get_job_id(&self) -> ResourceId {
        if self.job_id.is_none() {
            Default::default()
        } else {
            self.job_id.as_ref().unwrap().clone()
        }
    }

    #[inline]
    pub fn get_job_id_opt_ref(&self) -> Option<&ResourceId> {
        self.job_id.as_ref()
    }

    #[inline]
    pub fn get_key(&self) -> Entry {
        self.key.as_ref().map(|key| key.clone()).unwrap_or_default()
    }

    #[inline]
    pub fn get_event_time(&self) -> i64 {
        self.event_time
    }
}

impl FixedWindow {
    pub fn get_size(&self) -> Time {
        self.size
            .as_ref()
            .map(|size| size.clone())
            .unwrap_or_default()
    }
}

impl Watermark {
    pub fn get_trigger_time(&self) -> Time {
        self.trigger_time
            .as_ref()
            .map(|t| t.clone())
            .unwrap_or_default()
    }
}

impl SlidingWindow {
    pub fn get_size(&self) -> Time {
        self.size
            .as_ref()
            .map(|size| size.clone())
            .unwrap_or_default()
    }

    pub fn get_period(&self) -> Time {
        self.period
            .as_ref()
            .map(|period| period.clone())
            .unwrap_or_default()
    }
}

impl SessionWindow {
    pub fn get_timeout(&self) -> Time {
        self.timeout
            .as_ref()
            .map(|timeout| timeout.clone())
            .unwrap_or_default()
    }
}

impl Time {
    #[inline]
    pub fn to_duration(&self) -> Duration {
        let secs = (self.hours * 3600) as u64 + (self.minutes * 60) as u64 + self.seconds;
        Duration::seconds(secs as i64)
            .checked_add(&Duration::milliseconds(self.millis as i64))
            .unwrap_or(Duration::max_value())
    }
}

impl mysql_desc::ConnectionOpts {
    pub fn get_uri(&self) -> String {
        let db = &self.database;
        let user = &self.username;
        let password = &self.password;
        let host = &self.host;

        format!("mysql://{user}:{password}@{host}/{db}")
    }
}

impl Response {
    pub fn ok() -> Self {
        Self {
            status: SUCCESS_RPC_RESPONSE.to_string(),
            err_msg: String::default(),
        }
    }
}

impl SubDataflowId {
    pub fn get_job_id(&self) -> ResourceId {
        self.job_id
            .as_ref()
            .map(|job_id| job_id.clone())
            .unwrap_or_default()
    }
}

impl Ord for SubDataflowId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let job_id_order = self.job_id.cmp(&other.job_id);
        let sub_id_order = self.sub_id.cmp(&other.sub_id);
        job_id_order.then(sub_id_order)
    }
}

macro_rules! get_func {
    ($name:ident,$import:ident) => {
        use crate::common::{$import, $name};
        impl $name {
            pub fn get_func(&self) -> Func {
                match &self.value {
                    Some(value) => match value {
                        $import::Value::Func(func) => func.clone(),
                    },
                    None => Default::default(),
                }
            }
        }
    };
}

get_func!(FlatMap, flat_map);
get_func!(Mapper, mapper);
get_func!(Reducer, reducer);
get_func!(KeyBy, key_by);
get_func!(Filter, filter);

impl HostAddr {
    pub fn as_uri(&self) -> String {
        format!("http://{}:{}", &self.host, self.port)
    }

    pub fn is_valid(&self) -> bool {
        !self.host.is_empty() && self.port > 0
    }
}

impl Heartbeat {
    #[inline]
    pub fn get_subdataflow_id(&self) -> Option<&SubDataflowId> {
        self.subdataflow_id.as_ref()
    }
}

impl Ack {
    #[inline]
    pub fn get_execution_id(&self) -> Option<&SubDataflowId> {
        self.execution_id.as_ref()
    }
}

impl KeyedDataEvent {
    pub fn as_bytes(self) -> Result<bytes::Bytes, KeyedDataEventError> {
        apache_avro::Schema::parse_list(&[
            RESOURCE_ID_SCHEMA,
            WINDOW_SCHEMA,
            ENTRY_SCHEMA,
            KEYED_DATA_EVENT_SCHEMA,
        ])
        .and_then(|schemas| {
            let mut writer = apache_avro::Writer::new(&schemas[3], vec![]);

            writer
                .append_ser(self)
                .and_then(|_| writer.into_inner().map(|buf| bytes::Bytes::from(buf)))
        })
        .map_err(|err| KeyedDataEventError::AvroError(err))
    }

    pub fn from_slice(buf: &[u8]) -> Result<Self, KeyedDataEventError> {
        apache_avro::Schema::parse_list(&[
            RESOURCE_ID_SCHEMA,
            WINDOW_SCHEMA,
            ENTRY_SCHEMA,
            KEYED_DATA_EVENT_SCHEMA,
        ])
        .and_then(|schemas| {
            apache_avro::Reader::with_schema(&schemas[3], buf).and_then(|reader| {
                for value in reader {
                    match value {
                        Ok(val) => return apache_avro::from_value::<KeyedDataEvent>(&val),
                        Err(err) => return Err(err),
                    }
                }

                Ok(Default::default())
            })
        })
        .map_err(|err| KeyedDataEventError::AvroError(err))
    }
}

pub enum KeyedDataEventError {
    AvroError(apache_avro::Error),
}

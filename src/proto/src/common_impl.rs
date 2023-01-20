use std::hash::Hash;

use chrono::Duration;
use tonic::async_trait;

use crate::common::{
    mysql_desc::{self, Statement},
    operator_info::Details,
    sink, source,
    trigger::Watermark,
    window::{self, FixedWindow, SessionWindow, SlidingWindow},
    Ack, DataTypeEnum, Dataflow, Entry, ExecutionId, Func, Heartbeat, HostAddr, KafkaDesc,
    KeyedDataEvent, MysqlDesc, OperatorInfo, RedisDesc, ResourceId, Response, Sink, Source, Time,
    Trigger, Window,
};

pub const SUCCESS_RPC_RESPONSE: &str = "success";

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
    pub fn set_job_id(&mut self, resource_id: ResourceId) {
        self.job_id = Some(resource_id)
    }

    pub fn get_job_id(&self) -> ResourceId {
        if self.job_id.is_none() {
            Default::default()
        } else {
            self.job_id.as_ref().unwrap().clone()
        }
    }

    pub fn get_key(&self) -> Entry {
        self.key.as_ref().map(|key| key.clone()).unwrap_or_default()
    }

    pub fn get_event_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.event_time
            .as_ref()
            .map(|event_time| {
                chrono::NaiveDateTime::from_timestamp(event_time.seconds, event_time.nanos as u32)
            })
            .map(|datetime| chrono::DateTime::from_utc(datetime, chrono::Utc))
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

impl Hash for ResourceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.resource_id.hash(state);
        self.namespace_id.hash(state);
    }
}

impl PartialOrd for ResourceId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.resource_id.partial_cmp(&other.resource_id) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.namespace_id.partial_cmp(&other.namespace_id)
    }
}

impl Eq for ResourceId {}

impl Ord for ResourceId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(order) => order,
            None => std::cmp::Ordering::Equal,
        }
    }
}

impl serde::Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> serde::Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
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

impl ExecutionId {
    pub fn get_job_id(&self) -> ResourceId {
        self.job_id
            .as_ref()
            .map(|job_id| job_id.clone())
            .unwrap_or_default()
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
}

pub trait RpcGateway: Unpin {
    fn get_host_addr(&self) -> &HostAddr;
}

#[async_trait]
pub trait ReceiveAckRpcGateway: RpcGateway {
    async fn receive_ack(&self, req: Ack) -> Result<Response, tonic::Status>;
}

#[async_trait]
pub trait ReceiveHeartbeatRpcGateway: RpcGateway {
    async fn receive_heartbeat(&self, request: Heartbeat) -> Result<Response, tonic::Status>;
}

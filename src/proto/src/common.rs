#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProbeRequest {
    #[prost(enumeration = "probe_request::NodeType", tag = "1")]
    pub node_type: i32,
    #[prost(enumeration = "probe_request::ProbeType", tag = "2")]
    pub probe_type: i32,
}
/// Nested message and enum types in `ProbeRequest`.
pub mod probe_request {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum NodeType {
        Coordinator = 0,
        TaskWorker = 1,
        Connector = 2,
    }
    impl NodeType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                NodeType::Coordinator => "Coordinator",
                NodeType::TaskWorker => "TaskWorker",
                NodeType::Connector => "Connector",
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum ProbeType {
        Liveness = 0,
        Readiness = 1,
    }
    impl ProbeType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ProbeType::Liveness => "Liveness",
                ProbeType::Readiness => "Readiness",
            }
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProbeResponse {
    #[prost(float, tag = "1")]
    pub memory: f32,
    #[prost(float, tag = "2")]
    pub cpu: f32,
    #[prost(bool, tag = "3")]
    pub available: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventResponse {
    #[prost(int32, tag = "1")]
    pub code: i32,
    #[prost(string, tag = "2")]
    pub msg: ::prost::alloc::string::String,
}
/// *
/// JobId, represents a stream job.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceId {
    #[prost(string, tag = "1")]
    pub resource_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub namespace_id: ::prost::alloc::string::String,
}
/// common Rpc Response
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub err_msg: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HostAddr {
    #[prost(string, tag = "1")]
    pub host: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub port: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Time {
    #[prost(uint64, tag = "1")]
    pub millis: u64,
    #[prost(uint64, tag = "2")]
    pub seconds: u64,
    #[prost(uint32, tag = "3")]
    pub minutes: u32,
    #[prost(uint32, tag = "4")]
    pub hours: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataTypeEnum {
    Unspecified = 0,
    Bigint = 1,
    Number = 2,
    Null = 3,
    String = 4,
    Boolean = 5,
    Object = 6,
    Array = 7,
}
impl DataTypeEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataTypeEnum::Unspecified => "DATA_TYPE_ENUM_UNSPECIFIED",
            DataTypeEnum::Bigint => "DATA_TYPE_ENUM_BIGINT",
            DataTypeEnum::Number => "DATA_TYPE_ENUM_NUMBER",
            DataTypeEnum::Null => "DATA_TYPE_ENUM_NULL",
            DataTypeEnum::String => "DATA_TYPE_ENUM_STRING",
            DataTypeEnum::Boolean => "DATA_TYPE_ENUM_BOOLEAN",
            DataTypeEnum::Object => "DATA_TYPE_ENUM_OBJECT",
            DataTypeEnum::Array => "DATA_TYPE_ENUM_ARRAY",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ErrorCode {
    Unspecified = 0,
    ResourceNotFound = 1,
    RpcUnimplemented = 2,
    RpcUnavailable = 3,
    RpcUnauthorized = 4,
    RpcInvalidArgument = 5,
    RpcPermissionDenied = 6,
    InternalError = 7,
    AlreadyExists = 8,
    ResourceExhausted = 9,
    FailedPrecondition = 10,
    Aborted = 11,
    OutOfRange = 12,
    Cancelled = 13,
    Unknown = 14,
    DataflowOperatorInfoMissing = 15,
    CyclicDataflow = 16,
}
impl ErrorCode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ErrorCode::Unspecified => "ERROR_CODE_UNSPECIFIED",
            ErrorCode::ResourceNotFound => "ERROR_CODE_RESOURCE_NOT_FOUND",
            ErrorCode::RpcUnimplemented => "ERROR_CODE_RPC_UNIMPLEMENTED",
            ErrorCode::RpcUnavailable => "ERROR_CODE_RPC_UNAVAILABLE",
            ErrorCode::RpcUnauthorized => "ERROR_CODE_RPC_UNAUTHORIZED",
            ErrorCode::RpcInvalidArgument => "ERROR_CODE_RPC_INVALID_ARGUMENT",
            ErrorCode::RpcPermissionDenied => "ERROR_CODE_RPC_PERMISSION_DENIED",
            ErrorCode::InternalError => "ERROR_CODE_INTERNAL_ERROR",
            ErrorCode::AlreadyExists => "ERROR_CODE_ALREADY_EXISTS",
            ErrorCode::ResourceExhausted => "ERROR_CODE_RESOURCE_EXHAUSTED",
            ErrorCode::FailedPrecondition => "ERROR_CODE_FAILED_PRECONDITION",
            ErrorCode::Aborted => "ERROR_CODE_ABORTED",
            ErrorCode::OutOfRange => "ERROR_CODE_OUT_OF_RANGE",
            ErrorCode::Cancelled => "ERROR_CODE_CANCELLED",
            ErrorCode::Unknown => "ERROR_CODE_UNKNOWN",
            ErrorCode::DataflowOperatorInfoMissing => {
                "ERROR_CODE_DATAFLOW_OPERATOR_INFO_MISSING"
            }
            ErrorCode::CyclicDataflow => "ERROR_CODE_CYCLIC_DATAFLOW",
        }
    }
}
/// *
/// StreamGraph metadata, it stores the structural information of a stream graph
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataflowMeta {
    /// center node id
    #[prost(uint32, tag = "1")]
    pub center: u32,
    /// center's neighbors
    #[prost(uint32, repeated, tag = "2")]
    pub neighbors: ::prost::alloc::vec::Vec<u32>,
}
/// *
/// OperatorInfo, stores detail information of an operator
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperatorInfo {
    #[prost(uint32, tag = "1")]
    pub operator_id: u32,
    /// host addr configs
    #[prost(message, optional, tag = "2")]
    pub host_addr: ::core::option::Option<HostAddr>,
    /// upstreams operator_id
    #[prost(uint32, repeated, tag = "3")]
    pub upstreams: ::prost::alloc::vec::Vec<u32>,
    /// optional for different operator type
    #[prost(oneof = "operator_info::Details", tags = "5, 6, 7, 8, 9, 10, 11, 12, 13")]
    pub details: ::core::option::Option<operator_info::Details>,
}
/// Nested message and enum types in `OperatorInfo`.
pub mod operator_info {
    /// optional for different operator type
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Details {
        /// for source
        #[prost(message, tag = "5")]
        Source(super::Source),
        /// for sink
        #[prost(message, tag = "6")]
        Sink(super::Sink),
        #[prost(message, tag = "7")]
        Mapper(super::Mapper),
        #[prost(message, tag = "8")]
        Filter(super::Filter),
        #[prost(message, tag = "9")]
        KeyBy(super::KeyBy),
        #[prost(message, tag = "10")]
        Reducer(super::Reducer),
        #[prost(message, tag = "11")]
        FlatMap(super::FlatMap),
        #[prost(message, tag = "12")]
        Window(super::Window),
        ///     Join join = 11;
        #[prost(message, tag = "13")]
        Trigger(super::Trigger),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reducer {
    #[prost(oneof = "reducer::Value", tags = "1")]
    pub value: ::core::option::Option<reducer::Value>,
}
/// Nested message and enum types in `Reducer`.
pub mod reducer {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Func(super::Func),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlatMap {
    #[prost(oneof = "flat_map::Value", tags = "1")]
    pub value: ::core::option::Option<flat_map::Value>,
}
/// Nested message and enum types in `FlatMap`.
pub mod flat_map {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Func(super::Func),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Join {
    #[prost(oneof = "join::Value", tags = "1")]
    pub value: ::core::option::Option<join::Value>,
}
/// Nested message and enum types in `Join`.
pub mod join {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct StreamJoin {
        #[prost(uint32, tag = "1")]
        pub operator_id: u32,
        #[prost(message, optional, tag = "2")]
        pub func: ::core::option::Option<super::Func>,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        StreamJoin(StreamJoin),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mapper {
    #[prost(oneof = "mapper::Value", tags = "1")]
    pub value: ::core::option::Option<mapper::Value>,
}
/// Nested message and enum types in `Mapper`.
pub mod mapper {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Func(super::Func),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Func {
    #[prost(string, tag = "1")]
    pub function: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Filter {
    #[prost(oneof = "filter::Value", tags = "1")]
    pub value: ::core::option::Option<filter::Value>,
}
/// Nested message and enum types in `Filter`.
pub mod filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Func(super::Func),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyBy {
    #[prost(oneof = "key_by::Value", tags = "1")]
    pub value: ::core::option::Option<key_by::Value>,
}
/// Nested message and enum types in `KeyBy`.
pub mod key_by {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Func(super::Func),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sink {
    #[prost(oneof = "sink::Desc", tags = "1, 2, 3")]
    pub desc: ::core::option::Option<sink::Desc>,
}
/// Nested message and enum types in `Sink`.
pub mod sink {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Desc {
        #[prost(message, tag = "1")]
        Kafka(super::KafkaDesc),
        #[prost(message, tag = "2")]
        Mysql(super::MysqlDesc),
        #[prost(message, tag = "3")]
        Redis(super::RedisDesc),
    }
}
/// *
/// Constant operator
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConstOp {
    /// value of constant, format: [<flag byte>, <data bytes>]
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// operator id
    #[prost(uint32, tag = "2")]
    pub operator_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Source {
    #[prost(oneof = "source::Desc", tags = "3")]
    pub desc: ::core::option::Option<source::Desc>,
}
/// Nested message and enum types in `Source`.
pub mod source {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Desc {
        #[prost(message, tag = "3")]
        Kafka(super::KafkaDesc),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KafkaDesc {
    #[prost(string, repeated, tag = "1")]
    pub brokers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "2")]
    pub topic: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub opts: ::core::option::Option<kafka_desc::KafkaOptions>,
    #[prost(enumeration = "DataTypeEnum", tag = "4")]
    pub data_type: i32,
}
/// Nested message and enum types in `KafkaDesc`.
pub mod kafka_desc {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct KafkaOptions {
        #[prost(string, optional, tag = "1")]
        pub group: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(uint32, optional, tag = "2")]
        pub partition: ::core::option::Option<u32>,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MysqlDesc {
    #[prost(message, optional, tag = "1")]
    pub connection_opts: ::core::option::Option<mysql_desc::ConnectionOpts>,
    #[prost(message, optional, tag = "2")]
    pub statement: ::core::option::Option<mysql_desc::Statement>,
}
/// Nested message and enum types in `MysqlDesc`.
pub mod mysql_desc {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ConnectionOpts {
        #[prost(string, tag = "1")]
        pub host: ::prost::alloc::string::String,
        #[prost(string, tag = "2")]
        pub username: ::prost::alloc::string::String,
        #[prost(string, tag = "3")]
        pub password: ::prost::alloc::string::String,
        #[prost(string, tag = "4")]
        pub database: ::prost::alloc::string::String,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Statement {
        #[prost(string, tag = "1")]
        pub statement: ::prost::alloc::string::String,
        #[prost(message, repeated, tag = "2")]
        pub extractors: ::prost::alloc::vec::Vec<statement::Extractor>,
    }
    /// Nested message and enum types in `Statement`.
    pub mod statement {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Extractor {
            #[prost(uint32, tag = "1")]
            pub index: u32,
            #[prost(string, tag = "2")]
            pub extractor: ::prost::alloc::string::String,
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedisDesc {
    #[prost(message, optional, tag = "1")]
    pub connection_opts: ::core::option::Option<redis_desc::ConnectionOpts>,
    #[prost(message, optional, tag = "2")]
    pub key_extractor: ::core::option::Option<Func>,
    #[prost(message, optional, tag = "3")]
    pub value_extractor: ::core::option::Option<Func>,
}
/// Nested message and enum types in `RedisDesc`.
pub mod redis_desc {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ConnectionOpts {
        #[prost(string, tag = "1")]
        pub host: ::prost::alloc::string::String,
        #[prost(string, tag = "2")]
        pub username: ::prost::alloc::string::String,
        #[prost(string, tag = "3")]
        pub password: ::prost::alloc::string::String,
        #[prost(int64, tag = "4")]
        pub database: i64,
        #[prost(bool, tag = "5")]
        pub tls: bool,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Dataflow {
    /// job id, for now it is which table the stream graph output will sink
    #[prost(message, optional, tag = "1")]
    pub job_id: ::core::option::Option<ResourceId>,
    /// graph structure
    #[prost(message, repeated, tag = "2")]
    pub meta: ::prost::alloc::vec::Vec<DataflowMeta>,
    /// details of nodes
    #[prost(map = "uint32, message", tag = "3")]
    pub nodes: ::std::collections::HashMap<u32, OperatorInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Window {
    #[prost(message, optional, tag = "4")]
    pub trigger: ::core::option::Option<Trigger>,
    #[prost(oneof = "window::Value", tags = "1, 2, 3")]
    pub value: ::core::option::Option<window::Value>,
}
/// Nested message and enum types in `Window`.
pub mod window {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FixedWindow {
        /// Only for sliding & fixed window
        #[prost(message, optional, tag = "1")]
        pub size: ::core::option::Option<super::Time>,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SlidingWindow {
        #[prost(message, optional, tag = "1")]
        pub size: ::core::option::Option<super::Time>,
        #[prost(message, optional, tag = "2")]
        pub period: ::core::option::Option<super::Time>,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SessionWindow {
        /// Only for Session Window
        #[prost(message, optional, tag = "1")]
        pub timeout: ::core::option::Option<super::Time>,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Fixed(FixedWindow),
        #[prost(message, tag = "2")]
        Slide(SlidingWindow),
        #[prost(message, tag = "3")]
        Session(SessionWindow),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Trigger {
    #[prost(oneof = "trigger::Value", tags = "1")]
    pub value: ::core::option::Option<trigger::Value>,
}
/// Nested message and enum types in `Trigger`.
pub mod trigger {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Watermark {
        #[prost(message, optional, tag = "1")]
        pub trigger_time: ::core::option::Option<super::Time>,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Watermark(Watermark),
    }
}
/// *
/// Stream Graph Status. It shows which status a stream job is now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataflowStatus {
    Initialized = 0,
    Running = 1,
    Closing = 2,
    Closed = 3,
}
impl DataflowStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataflowStatus::Initialized => "INITIALIZED",
            DataflowStatus::Running => "RUNNING",
            DataflowStatus::Closing => "CLOSING",
            DataflowStatus::Closed => "CLOSED",
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyedDataEvent {
    /// source id
    #[prost(message, optional, tag = "1")]
    pub job_id: ::core::option::Option<ResourceId>,
    /// key of data event
    #[prost(message, optional, tag = "2")]
    pub key: ::core::option::Option<Entry>,
    /// operator_id this event will be sent
    #[prost(uint32, tag = "3")]
    pub to_operator_id: u32,
    /// mandatory
    #[prost(message, repeated, tag = "5")]
    pub data: ::prost::alloc::vec::Vec<Entry>,
    #[prost(message, optional, tag = "6")]
    pub event_time: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag = "7")]
    pub process_time: ::core::option::Option<::prost_types::Timestamp>,
    /// operator_id this event where be sent
    #[prost(uint32, tag = "8")]
    pub from_operator_id: u32,
    #[prost(message, optional, tag = "9")]
    pub window: ::core::option::Option<keyed_data_event::Window>,
}
/// Nested message and enum types in `KeyedDataEvent`.
pub mod keyed_data_event {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Window {
        #[prost(message, optional, tag = "1")]
        pub start_time: ::core::option::Option<::prost_types::Timestamp>,
        #[prost(message, optional, tag = "2")]
        pub end_time: ::core::option::Option<::prost_types::Timestamp>,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Entry {
    #[prost(enumeration = "DataTypeEnum", tag = "1")]
    pub data_type: i32,
    /// entry value
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}

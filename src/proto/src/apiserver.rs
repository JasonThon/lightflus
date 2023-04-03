#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceRequest {
    /// namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// resource type to create
    #[prost(enumeration = "ResourceTypeEnum", tag = "2")]
    pub resource_type: i32,
    #[prost(oneof = "create_resource_request::Options", tags = "3")]
    pub options: ::core::option::Option<create_resource_request::Options>,
}
/// Nested message and enum types in `CreateResourceRequest`.
pub mod create_resource_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Options {
        #[prost(message, tag = "3")]
        Dataflow(super::CreateDataflowOptions),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResourceResponse {
    #[prost(enumeration = "ResourceStatusEnum", tag = "1")]
    pub status: i32,
    #[prost(message, optional, tag = "2")]
    pub resource_id: ::core::option::Option<super::common::ResourceId>,
    #[prost(string, tag = "3")]
    pub error_msg: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDataflowOptions {
    #[prost(message, optional, tag = "1")]
    pub dataflow: ::core::option::Option<super::common::Dataflow>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResourcesRequest {
    /// namespace
    #[prost(string, tag = "1")]
    pub namespace: ::prost::alloc::string::String,
    /// resource type to list
    #[prost(enumeration = "ResourceTypeEnum", tag = "2")]
    pub resource_type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResourcesResponse {
    #[prost(message, repeated, tag = "1")]
    pub resources: ::prost::alloc::vec::Vec<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Resource {
    #[prost(message, optional, tag = "1")]
    pub resource_id: ::core::option::Option<super::common::ResourceId>,
    #[prost(string, tag = "2")]
    pub resource_name: ::prost::alloc::string::String,
    /// resource type
    #[prost(enumeration = "ResourceTypeEnum", tag = "3")]
    pub resource_type: i32,
    #[prost(enumeration = "ResourceStatusEnum", tag = "4")]
    pub status: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourceRequest {
    #[prost(message, optional, tag = "1")]
    pub resource_id: ::core::option::Option<super::common::ResourceId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResourceResponse {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Resource>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceRequest {
    #[prost(message, optional, tag = "1")]
    pub resource_id: ::core::option::Option<super::common::ResourceId>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResourceResponse {
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Resource>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResourceTypeEnum {
    Unspecific = 0,
    Dataflow = 1,
}
impl ResourceTypeEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ResourceTypeEnum::Unspecific => "RESOURCE_TYPE_ENUM_UNSPECIFIC",
            ResourceTypeEnum::Dataflow => "RESOURCE_TYPE_ENUM_DATAFLOW",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RESOURCE_TYPE_ENUM_UNSPECIFIC" => Some(Self::Unspecific),
            "RESOURCE_TYPE_ENUM_DATAFLOW" => Some(Self::Dataflow),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResourceStatusEnum {
    Unspecific = 0,
    Starting = 1,
    Running = 2,
    Failure = 3,
    Stopping = 4,
    Deleted = 5,
}
impl ResourceStatusEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ResourceStatusEnum::Unspecific => "RESOURCE_STATUS_ENUM_UNSPECIFIC",
            ResourceStatusEnum::Starting => "RESOURCE_STATUS_ENUM_STARTING",
            ResourceStatusEnum::Running => "RESOURCE_STATUS_ENUM_RUNNING",
            ResourceStatusEnum::Failure => "RESOURCE_STATUS_ENUM_FAILURE",
            ResourceStatusEnum::Stopping => "RESOURCE_STATUS_ENUM_STOPPING",
            ResourceStatusEnum::Deleted => "RESOURCE_STATUS_ENUM_DELETED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "RESOURCE_STATUS_ENUM_UNSPECIFIC" => Some(Self::Unspecific),
            "RESOURCE_STATUS_ENUM_STARTING" => Some(Self::Starting),
            "RESOURCE_STATUS_ENUM_RUNNING" => Some(Self::Running),
            "RESOURCE_STATUS_ENUM_FAILURE" => Some(Self::Failure),
            "RESOURCE_STATUS_ENUM_STOPPING" => Some(Self::Stopping),
            "RESOURCE_STATUS_ENUM_DELETED" => Some(Self::Deleted),
            _ => None,
        }
    }
}

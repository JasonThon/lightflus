use bytes::Buf;
use proto::common::common::{DataTypeEnum, ResourceId};
use proto::common::event::Entry;
use protobuf::ProtobufEnum;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::{ops, vec};

pub(crate) const STRING_SYMBOL: &str = "string";
pub(crate) const NUMBER_SYMBOL: &str = "number";
pub(crate) const NULL_SYMBOL: &str = "null";
pub(crate) const UNDEFINED_SYMBOL: &str = "undefined";
pub(crate) const BOOLEAN_SYMBOL: &str = "boolean";
pub(crate) const OBJECT_SYMBOL: &str = "object";
pub(crate) const BIGINT_SYMBOL: &str = "bigint";

impl PartialEq for TypedValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::BigInt(l0), Self::BigInt(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Object(l0), Self::Object(r0)) => l0 == r0,
            (Self::Null, Self::Null) => true,
            (Self::Invalid, Self::Invalid) => true,
            (Self::BigInt(l0), Self::Number(r0)) => (*l0 as f64) == *r0,
            (Self::Number(l0), Self::BigInt(r0)) => *l0 == (*r0 as f64),
            (Self::Null, Self::Invalid) => true,
            (Self::Invalid, Self::Null) => true,
            (Self::Array(l0), Self::Array(l1)) => l0 == l1,
            _ => false,
        }
    }
}

impl PartialOrd for TypedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0.partial_cmp(r0),
            (Self::BigInt(l0), Self::BigInt(r0)) => l0.partial_cmp(r0),
            (Self::Boolean(l0), Self::Boolean(r0)) => l0.partial_cmp(r0),
            (Self::Number(l0), Self::Number(r0)) => l0.partial_cmp(r0),
            (Self::Object(l0), Self::Object(r0)) => l0.partial_cmp(r0),
            (Self::Null, Self::Null) => Some(Ordering::Equal),
            (Self::Invalid, Self::Invalid) => Some(Ordering::Equal),
            (Self::BigInt(l0), Self::Number(r0)) => (*l0 as f64).partial_cmp(r0),
            (Self::Number(l0), Self::BigInt(r0)) => l0.partial_cmp(&(*r0 as f64)),
            (Self::Null, Self::Invalid) => Some(Ordering::Equal),
            (Self::Invalid, Self::Null) => Some(Ordering::Equal),
            (Self::Array(l0), Self::Array(l1)) => l0.partial_cmp(l1),
            _ => None,
        }
    }
}

// TODO fix float calculated with double precision loss problem
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum TypedValue {
    String(String),
    BigInt(i64),
    Boolean(bool),
    Number(f64),
    Null,
    Object(BTreeMap<String, Vec<u8>>),
    Array(Vec<TypedValue>),
    Invalid,
}

impl Eq for TypedValue {}
impl Ord for TypedValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(order) => order,
            None => Ordering::Equal,
        }
    }
}

impl Default for TypedValue {
    fn default() -> Self {
        Self::Invalid
    }
}

macro_rules! ops_helper {
    ($ops:ident, $func_name:ident) => {
        impl ops::$ops for TypedValue {
            type Output = TypedValue;

            fn $func_name(self, rhs: Self) -> Self::Output {
                match self {
                    TypedValue::Number(value) => match rhs {
                        TypedValue::Number(other) => TypedValue::Number(value.$func_name(other)),
                        TypedValue::BigInt(other) => {
                            TypedValue::Number(value.$func_name((other as f64)))
                        }
                        _ => TypedValue::Invalid,
                    },
                    TypedValue::Null => TypedValue::Null,
                    TypedValue::BigInt(value) => match rhs {
                        TypedValue::Number(other) => {
                            TypedValue::Number((value as f64).$func_name(other))
                        }
                        TypedValue::BigInt(other) => TypedValue::BigInt(value.$func_name(other)),
                        _ => TypedValue::Invalid,
                    },
                    _ => TypedValue::Invalid,
                }
            }
        }
    };
}

ops_helper!(Sub, sub);
ops_helper!(Div, div);
ops_helper!(Mul, mul);
ops_helper!(Add, add);

macro_rules! ops_assign_helper {
    ($ops_assign:ident, $func_name:ident) => {
        impl ops::$ops_assign for TypedValue {
            fn $func_name(&mut self, rhs: Self) {
                match self {
                    TypedValue::Number(value) => match rhs {
                        TypedValue::Number(other) => value.$func_name(other),
                        TypedValue::BigInt(other) => value.$func_name(other as f64),
                        _ => {}
                    },
                    TypedValue::BigInt(value) => match rhs {
                        TypedValue::BigInt(other) => value.$func_name(other),
                        TypedValue::Number(other) => (*value as f64).$func_name(other),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    };
}

ops_assign_helper!(MulAssign, mul_assign);
ops_assign_helper!(SubAssign, sub_assign);
ops_assign_helper!(DivAssign, div_assign);
ops_assign_helper!(AddAssign, add_assign);

impl TypedValue {
    pub fn get_data(&self) -> Vec<u8> {
        let data_type = self.get_type();
        let mut result = vec![data_type as u8];
        let ref mut raw_data = match self {
            TypedValue::String(value) => value.as_bytes().to_vec(),
            TypedValue::Number(value) => value.to_be_bytes().to_vec(),
            TypedValue::BigInt(value) => value.to_be_bytes().to_vec(),
            TypedValue::Boolean(value) => {
                let data = if *value { 1 as u8 } else { 0 as u8 };
                vec![data]
            }
            TypedValue::Object(value) => serde_json::to_vec(value)
                .map_err(|err| log::error!("serialize object failed: {}", err))
                .unwrap_or_default(),
            TypedValue::Array(value) => {
                let result = Vec::from_iter(value.iter().map(|value| value.get_data()));
                serde_json::to_vec(&result)
                    .map_err(|err| log::error!("serialize array failed: {}", err))
                    .unwrap_or_default()
            }
            _ => vec![],
        };
        result.append(raw_data);

        result
    }

    pub fn from_vec(data: &Vec<u8>) -> Self {
        let data_type = DataTypeEnum::from_i32(data[0] as i32).unwrap_or_default();

        match data_type {
            DataTypeEnum::DATA_TYPE_ENUM_STRING => {
                match String::from_utf8(data[1..data.len()].to_vec()) {
                    Ok(val) => TypedValue::String(val),
                    Err(_) => TypedValue::Invalid,
                }
            }
            DataTypeEnum::DATA_TYPE_ENUM_BIGINT => {
                TypedValue::BigInt(data[1..data.len()].to_vec().as_slice().get_i64())
            }
            DataTypeEnum::DATA_TYPE_ENUM_NUMBER => {
                TypedValue::Number(data[1..data.len()].to_vec().as_slice().get_f64())
            }
            DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN => TypedValue::Boolean(data[1] == 1),
            DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED => TypedValue::Invalid,
            DataTypeEnum::DATA_TYPE_ENUM_NULL => TypedValue::Null,
            DataTypeEnum::DATA_TYPE_ENUM_OBJECT => TypedValue::Object(
                serde_json::from_slice::<BTreeMap<String, Vec<u8>>>(&data[1..data.len()])
                    .map_err(|err| log::error!("deserialize object failed: {}", err))
                    .unwrap_or_default(),
            ),
            DataTypeEnum::DATA_TYPE_ENUM_ARRAY => {
                let val = serde_json::from_slice::<Vec<Vec<u8>>>(&data[1..data.len()])
                    .map_err(|err| log::error!("deserializ array failed:{}", err))
                    .unwrap_or_default();
                TypedValue::Array(Vec::from_iter(
                    val.iter().map(|data| TypedValue::from_vec(data)),
                ))
            }
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let data_type = DataTypeEnum::from_i32(data[0] as i32).unwrap_or_default();

        match data_type {
            DataTypeEnum::DATA_TYPE_ENUM_STRING => {
                match String::from_utf8(data[1..data.len()].to_vec()) {
                    Ok(val) => TypedValue::String(val),
                    Err(_) => TypedValue::Invalid,
                }
            }
            DataTypeEnum::DATA_TYPE_ENUM_BIGINT => {
                TypedValue::BigInt(data[1..data.len()].to_vec().as_slice().get_i64())
            }
            DataTypeEnum::DATA_TYPE_ENUM_NUMBER => {
                TypedValue::Number(data[1..data.len()].to_vec().as_slice().get_f64())
            }
            DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN => TypedValue::Boolean(data[1] == 1),
            DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED => TypedValue::Invalid,
            DataTypeEnum::DATA_TYPE_ENUM_NULL => TypedValue::Null,
            DataTypeEnum::DATA_TYPE_ENUM_OBJECT => TypedValue::Object(
                serde_json::from_slice::<BTreeMap<String, Vec<u8>>>(data)
                    .unwrap_or(Default::default()),
            ),
            DataTypeEnum::DATA_TYPE_ENUM_ARRAY => {
                let val = serde_json::from_slice::<Vec<Vec<u8>>>(data).unwrap_or_default();
                TypedValue::Array(Vec::from_iter(
                    val.iter().map(|data| TypedValue::from_vec(data)),
                ))
            }
        }
    }

    pub fn get_type(&self) -> DataTypeEnum {
        match self {
            TypedValue::String(_) => DataTypeEnum::DATA_TYPE_ENUM_STRING,
            TypedValue::Number(_) => DataTypeEnum::DATA_TYPE_ENUM_NUMBER,
            TypedValue::BigInt(_) => DataTypeEnum::DATA_TYPE_ENUM_BIGINT,
            TypedValue::Null => DataTypeEnum::DATA_TYPE_ENUM_NULL,
            TypedValue::Object(_) => DataTypeEnum::DATA_TYPE_ENUM_OBJECT,
            TypedValue::Boolean(_) => DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN,
            TypedValue::Array(_) => DataTypeEnum::DATA_TYPE_ENUM_ARRAY,
            _ => DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED,
        }
    }
}

impl From<&Entry> for TypedValue {
    fn from(entry: &Entry) -> Self {
        let mut data = vec![];
        data.extend_from_slice(entry.get_value());
        Self::from_vec(&data)
    }
}

pub type RowIdx = u64;
pub type NodeIdx = u32;
pub type SinkId = u32;
pub type SourceId = u32;
pub type ExecutorId = u32;

pub trait FromBytes: Sized {
    fn from_bytes(data: Vec<u8>) -> Option<Self>;
    fn to_string(&self) -> String;
}

pub trait KeyedValue<K, V> {
    fn key(&self) -> K;
    fn value(&self) -> V;
}

#[derive(Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct HashedResourceId {
    pub stream_id: String,
}

impl From<ResourceId> for HashedResourceId {
    fn from(id: ResourceId) -> Self {
        Self {
            stream_id: id.resource_id,
        }
    }
}

pub struct SingleKV<K> {
    key: K,
}

impl<K> SingleKV<K> {
    pub fn new(key: K) -> Self {
        Self { key }
    }
}

impl<K> KeyedValue<K, K> for SingleKV<K>
where
    K: Hash + Clone,
{
    fn key(&self) -> K {
        self.key.clone()
    }

    fn value(&self) -> K {
        self.key.clone()
    }
}

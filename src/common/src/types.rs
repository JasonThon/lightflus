use bytes::{Buf, BufMut};
use proto::common::common::{DataTypeEnum, ResourceId};
use proto::common::event::Entry;
use protobuf::ProtobufEnum;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::ops;

pub(crate) const STRING_SYMBOL: &str = "string";
pub(crate) const NUMBER_SYMBOL: &str = "number";
pub(crate) const NULL_SYMBOL: &str = "null";
pub(crate) const UNDEFINED_SYMBOL: &str = "undefined";
pub(crate) const BOOLEAN_SYMBOL: &str = "boolean";
pub(crate) const OBJECT_SYMBOL: &str = "object";
pub(crate) const BIGINT_SYMBOL: &str = "bigint";

// TODO fix float calculated with double precision loss problem
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum TypedValue {
    String(String),
    BigInt(i64),
    Boolean(bool),
    Number(f64),
    Null,
    Object(BTreeMap<String, Vec<u8>>),
    Invalid,
}

impl Eq for TypedValue {}

// TODO refactor by macro in future
impl PartialEq for TypedValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TypedValue::String(value) => match other {
                TypedValue::String(other) => value == other,
                _ => false,
            },
            TypedValue::Number(value) => match other {
                TypedValue::Number(other) => value.eq(other),
                _ => false,
            },
            TypedValue::Null => match other {
                TypedValue::Null => true,
                _ => false,
            },
            TypedValue::Object(value) => match other {
                TypedValue::Object(other) => value.eq(other),
                _ => false,
            },
            TypedValue::BigInt(value) => match other {
                TypedValue::Number(other) => (*value as f64).eq(other),
                TypedValue::BigInt(other) => value == other,
                _ => false,
            },
            TypedValue::Boolean(value) => match other {
                TypedValue::Boolean(other) => value == other,
                _ => false,
            },
            TypedValue::Invalid => match other {
                TypedValue::Invalid => true,
                _ => false,
            },
        }
    }
}

// TODO refactor by macro in future
impl PartialOrd for TypedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            TypedValue::String(value) => match other {
                TypedValue::String(other) => value.partial_cmp(other),
                _ => None,
            },
            TypedValue::Number(value) => match other {
                TypedValue::Number(other) => value.partial_cmp(other),
                TypedValue::BigInt(other) => value.partial_cmp(&(*other as f64)),
                _ => None,
            },
            TypedValue::Object(value) => match other {
                TypedValue::Object(other) => value.partial_cmp(other),
                _ => None,
            },
            TypedValue::Null => match other {
                TypedValue::Null => Some(Ordering::Equal),
                _ => None,
            },
            TypedValue::BigInt(value) => match other {
                TypedValue::Number(other) => (*value as f64).partial_cmp(other),
                TypedValue::BigInt(other) => value.partial_cmp(other),
                _ => None,
            },
            _ => None,
        }
    }
}

// TODO refactor by macro in future
impl ops::BitOr for TypedValue {
    type Output = TypedValue;

    fn bitor(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

// TODO refactor by macro in future
impl ops::BitAnd for TypedValue {
    type Output = TypedValue;

    fn bitand(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

// TODO refactor by macro in future
impl ops::Sub for TypedValue {
    type Output = TypedValue;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            TypedValue::Number(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number(value - other),
                TypedValue::BigInt(other) => TypedValue::Number(value - (other as f64)),
                _ => TypedValue::Invalid,
            },
            TypedValue::Null => TypedValue::Null,
            TypedValue::BigInt(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number((value as f64) - other),
                TypedValue::BigInt(other) => TypedValue::BigInt(value - other),
                _ => TypedValue::Invalid,
            },
            _ => TypedValue::Invalid,
        }
    }
}

// TODO refactor by macro in future
impl ops::Div for TypedValue {
    type Output = TypedValue;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            TypedValue::Number(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number(value / other),
                TypedValue::BigInt(other) => TypedValue::Number(value / (other as f64)),
                _ => TypedValue::Invalid,
            },
            TypedValue::Null => TypedValue::Null,
            TypedValue::BigInt(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number((value as f64) / other),
                TypedValue::BigInt(other) => TypedValue::Number((value as f64) / (other as f64)),
                _ => TypedValue::Invalid,
            },
            _ => TypedValue::Invalid,
        }
    }
}

// TODO refactor by macro in future
impl ops::MulAssign for TypedValue {
    fn mul_assign(&mut self, rhs: Self) {
        todo!()
    }
}

// TODO refactor by macro in future
impl ops::SubAssign for TypedValue {
    fn sub_assign(&mut self, rhs: Self) {
        todo!()
    }
}

// TODO refactor by macro in future
impl ops::DivAssign for TypedValue {
    fn div_assign(&mut self, rhs: Self) {
        todo!()
    }
}

// TODO refactor by macro in future
impl ops::Mul for TypedValue {
    type Output = TypedValue;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            TypedValue::Number(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number(other * value),
                TypedValue::BigInt(other) => TypedValue::Number(other as f64 * value),
                _ => TypedValue::Invalid,
            },
            TypedValue::Null => TypedValue::Null,
            TypedValue::BigInt(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number(other * value as f64),
                TypedValue::BigInt(other) => TypedValue::BigInt(other * value),
                _ => TypedValue::Invalid,
            },
            _ => TypedValue::Invalid,
        }
    }
}

// TODO refactor by macro in future
impl ops::Add for TypedValue {
    type Output = TypedValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            TypedValue::String(value) => match rhs {
                TypedValue::String(other) => TypedValue::String(value.add(other.as_str())),
                _ => TypedValue::Invalid,
            },
            TypedValue::Number(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number(value.add(other)),
                TypedValue::BigInt(other) => TypedValue::Number(value.add(other as f64)),
                _ => TypedValue::Invalid,
            },
            TypedValue::Null => self,
            TypedValue::BigInt(value) => match rhs {
                TypedValue::Number(other) => TypedValue::Number((value as f64) + other),
                TypedValue::BigInt(other) => TypedValue::BigInt(value + other),
                _ => TypedValue::Invalid,
            },
            _ => TypedValue::Invalid,
        }
    }
}

// TODO refactor by macro in future
impl ops::AddAssign for TypedValue {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            TypedValue::String(value) => match rhs {
                TypedValue::String(other) => value.push_str(other.as_str()),
                TypedValue::Number(other) => value.push_str(other.to_string().as_str()),
                TypedValue::BigInt(other) => value.push_str(other.to_string().as_str()),
                _ => {}
            },
            TypedValue::Number(value) => match rhs {
                TypedValue::Number(other) => value.add_assign(other),
                _ => {}
            },
            TypedValue::BigInt(value) => match rhs {
                TypedValue::BigInt(other) => value.add_assign(other),
                _ => {}
            },
            _ => {}
        }
    }
}

impl TypedValue {
    pub fn get_data(&self) -> Vec<u8> {
        match self {
            TypedValue::String(value) => value.as_bytes().to_vec(),
            TypedValue::Number(value) => value.to_be_bytes().to_vec(),
            TypedValue::BigInt(value) => value.to_be_bytes().to_vec(),
            TypedValue::Boolean(value) => {
                let data = if *value { 1 as u8 } else { 0 as u8 };
                vec![data]
            }
            _ => vec![],
        }
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
                serde_json::from_slice::<BTreeMap<String, Vec<u8>>>(data.as_slice())
                    .unwrap_or(Default::default()),
            ),
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
            _ => DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED,
        }
    }
}

impl From<&Entry> for TypedValue {
    fn from(entry: &Entry) -> Self {
        let symbol = entry.get_data_type().value() as u8;
        let mut data = vec![symbol];
        data.put_slice(entry.get_value());
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

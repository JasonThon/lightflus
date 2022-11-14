use bytes::Buf;
use proto::common::common::{DataTypeEnum, ResourceId};
use proto::common::event::Entry;
use protobuf::ProtobufEnum;
use redis::ToRedisArgs;
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
    Object(BTreeMap<String, TypedValue>),
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

impl ToRedisArgs for TypedValue {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            TypedValue::String(v) => out.write_arg(v.as_bytes()),
            TypedValue::BigInt(v) => out.write_arg(&v.to_be_bytes()),
            TypedValue::Boolean(v) => out.write_arg(&[*v as u8]),
            TypedValue::Number(v) => out.write_arg(&v.to_be_bytes()),
            TypedValue::Null => out.write_arg("null".as_bytes()),
            TypedValue::Object(v) => v
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone()))
                .collect::<BTreeMap<String, TypedValue>>()
                .write_redis_args(out),
            TypedValue::Array(v) => v.write_redis_args(out),
            TypedValue::Invalid => out.write_arg("null".as_bytes()),
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
                serde_json::from_slice::<BTreeMap<String, TypedValue>>(&data[1..data.len()])
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
                serde_json::from_slice::<BTreeMap<String, TypedValue>>(&data[1..data.len()])
                    .map_err(|err| log::error!("{err}"))
                    .unwrap_or(Default::default()),
            ),
            DataTypeEnum::DATA_TYPE_ENUM_ARRAY => {
                let val = serde_json::from_slice::<Vec<Vec<u8>>>(&data[1..data.len()])
                    .map_err(|err| log::error!("{err}"))
                    .unwrap_or_default();
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

    pub fn from_slice_with_type(mut data: &[u8], data_type: DataTypeEnum) -> Self {
        match data_type {
            DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED => Self::Invalid,
            DataTypeEnum::DATA_TYPE_ENUM_BIGINT => Self::BigInt(data.get_i64()),
            DataTypeEnum::DATA_TYPE_ENUM_NUMBER => Self::Number(data.get_f64()),
            DataTypeEnum::DATA_TYPE_ENUM_NULL => Self::Null,
            DataTypeEnum::DATA_TYPE_ENUM_STRING => {
                Self::String(String::from_utf8_lossy(data).to_string())
            }
            DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN => Self::Boolean(data[0] == 1),
            DataTypeEnum::DATA_TYPE_ENUM_OBJECT => {
                let value = serde_json::from_slice::<serde_json::Value>(data);
                match value {
                    Ok(val) => Self::from_json_value(val),
                    Err(err) => {
                        log::error!("deserialize json object failed: {}", err);
                        Self::Object(Default::default())
                    }
                }
            }
            DataTypeEnum::DATA_TYPE_ENUM_ARRAY => {
                let value = serde_json::from_slice::<serde_json::Value>(data);
                match value {
                    Ok(val) => Self::from_json_value(val),
                    Err(err) => {
                        log::error!("deserialize json array failed: {}", err);
                        Self::Array(Default::default())
                    }
                }
            }
        }
    }

    pub fn from_json_value(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(v) => Self::Boolean(v),
            serde_json::Value::Number(v) => {
                if v.is_f64() {
                    Self::Number(v.as_f64().unwrap())
                } else if v.is_i64() {
                    Self::BigInt(v.as_i64().unwrap())
                } else if v.is_u64() {
                    Self::BigInt(v.as_u64().unwrap() as i64)
                } else {
                    Self::Invalid
                }
            }
            serde_json::Value::String(v) => Self::String(v),
            serde_json::Value::Array(v) => Self::Array(
                v.iter()
                    .map(|value| Self::from_json_value(value.clone()))
                    .collect(),
            ),
            serde_json::Value::Object(v) => Self::Object(
                v.iter()
                    .map(|entry| (entry.0.clone(), Self::from_json_value(entry.1.clone())))
                    .collect(),
            ),
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            TypedValue::String(v) => serde_json::Value::String(v.clone()),
            TypedValue::BigInt(v) => serde_json::Value::Number(serde_json::Number::from(*v)),
            TypedValue::Boolean(v) => serde_json::Value::Bool(*v),
            TypedValue::Number(v) => serde_json::Value::Number(
                serde_json::Number::from_f64(*v).unwrap_or(serde_json::Number::from(0)),
            ),
            TypedValue::Null => serde_json::Value::Null,
            TypedValue::Object(v) => serde_json::Value::Object(
                v.iter()
                    .map(|entry| (entry.0.clone(), entry.1.to_json_value()))
                    .collect(),
            ),
            TypedValue::Array(v) => {
                serde_json::Value::Array(v.iter().map(|value| value.to_json_value()).collect())
            }
            TypedValue::Invalid => serde_json::Value::Null,
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

#[derive(Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
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

impl From<&ResourceId> for HashedResourceId {
    fn from(id: &ResourceId) -> Self {
        Self {
            stream_id: id.resource_id.clone(),
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

mod tests {

    #[test]
    fn test_typed_value_get_data() {
        use super::TypedValue;
        use bytes::Buf;
        use proto::common::common::DataTypeEnum;

        let int = TypedValue::BigInt(1 << 30);
        let mut data = int.get_data();
        let _ = data.remove(0);
        assert_eq!(data.as_slice().get_i64(), 1 << 30);
        assert_eq!(int.get_type(), DataTypeEnum::DATA_TYPE_ENUM_BIGINT);

        let double = super::TypedValue::Number(1.6546);
        let mut data = double.get_data();
        let _ = data.remove(0);
        assert_eq!(data.as_slice().get_f64(), 1.6546);
        assert_eq!(double.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NUMBER);

        let float = super::TypedValue::Null;
        assert_eq!(float.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NULL);
        let mut data = float.get_data();
        let _ = data.remove(0);
        assert_eq!(data.len(), 0);

        let string = super::TypedValue::String("test".to_string());
        assert_eq!(string.get_type(), DataTypeEnum::DATA_TYPE_ENUM_STRING);
        let mut data = string.get_data();
        let _ = data.remove(0);
        assert_eq!(String::from_utf8(data), Ok("test".to_string()))
    }

    #[test]
    fn test_typed_value_left_int_dual_op() {
        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::BigInt(200);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::BigInt(300));
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::BigInt(-100));
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::BigInt(20000));
        assert_eq!(a1 / a2, super::TypedValue::BigInt(0));

        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::BigInt(3000);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::BigInt(3100));
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::BigInt(-2900));
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::BigInt(300000));
        assert_eq!(a1 / a2, super::TypedValue::BigInt(0));

        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::Number(313.129);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Number(413.129));
        assert_eq!(
            a1.clone() - a2.clone(),
            super::TypedValue::Number(-213.12900000000002)
        );
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Number(31312.9));
        assert_eq!(a1 / a2, super::TypedValue::Number(0.31935719783220334));

        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::String("sss".to_string());
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1 / a2, super::TypedValue::Invalid);

        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::Boolean(true);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1 / a2, super::TypedValue::Invalid);

        let a1 = super::TypedValue::BigInt(100);
        let a2 = super::TypedValue::Invalid;
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1 / a2, super::TypedValue::Invalid);
    }

    #[test]
    fn test_typed_value_left_long_dual_op() {
        std::env::set_var("double.accuracy", 8.to_string());
        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::BigInt(2000);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::BigInt(3000));
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::BigInt(-1000));
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::BigInt(2000000));
        assert_eq!(a1 / a2, super::TypedValue::BigInt(0));

        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::Number(229.102);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Number(1229.102));
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Number(770.898));
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Number(229102.0));
        assert_eq!(a1 / a2, super::TypedValue::Number(4.364868050038847));

        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::Number(229.102);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Number(1229.102));
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Number(770.898));
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Number(229102.0));
        assert_eq!(a1 / a2, super::TypedValue::Number(4.364868050038847));

        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::String("sss".to_string());
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() / a2.clone(), super::TypedValue::Invalid);

        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::Boolean(true);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() / a2.clone(), super::TypedValue::Invalid);

        let a1 = super::TypedValue::BigInt(1000);
        let a2 = super::TypedValue::Invalid;
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() / a2.clone(), super::TypedValue::Invalid);
    }

    #[test]
    fn test_typed_value_left_float_dual_op() {
        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::String("sss".to_string());
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() / a2.clone(), super::TypedValue::Invalid);

        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::Boolean(true);
        assert_eq!(a1.clone() + a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() - a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() * a2.clone(), super::TypedValue::Invalid);
        assert_eq!(a1.clone() / a2.clone(), super::TypedValue::Invalid);

        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::Number(899.9999);
        assert_eq!(
            a1.clone() + a2.clone(),
            super::TypedValue::Number(2899.111122333)
        );
        assert_eq!(
            a1.clone() - a2.clone(),
            super::TypedValue::Number(1099.1113223329999)
        );
        assert_eq!(
            a1.clone() * a2.clone(),
            super::TypedValue::Number(1799199.9001885778)
        );
        assert_eq!(
            a1.clone() / a2.clone(),
            super::TypedValue::Number(2.2212349382849927)
        );

        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::BigInt(899);
        assert_eq!(
            a1.clone() + a2.clone(),
            super::TypedValue::Number(2898.111222333)
        );
        assert_eq!(
            a1.clone() - a2.clone(),
            super::TypedValue::Number(1100.111222333)
        );
        assert_eq!(
            a1.clone() * a2.clone(),
            super::TypedValue::Number(1797200.988877367)
        );
        assert_eq!(
            a1.clone() / a2.clone(),
            super::TypedValue::Number(2.223705475342603)
        );

        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::BigInt(899);
        assert_eq!(
            a1.clone() + a2.clone(),
            super::TypedValue::Number(2898.111222333)
        );
        assert_eq!(
            a1.clone() - a2.clone(),
            super::TypedValue::Number(1100.111222333)
        );
        assert_eq!(
            a1.clone() * a2.clone(),
            super::TypedValue::Number(1797200.988877367)
        );
        assert_eq!(
            a1.clone() / a2.clone(),
            super::TypedValue::Number(2.223705475342603)
        );

        let a1 = super::TypedValue::Number(1999.111222333);
        let a2 = super::TypedValue::Number(899.99099);
        // TODO float calculate with double should not loss precision
        assert_eq!(
            a1.clone() + a2.clone(),
            super::TypedValue::Number(2899.102212333)
        );
        assert_eq!(
            a1.clone() - a2.clone(),
            super::TypedValue::Number(1099.1202323329999)
        );
        assert_eq!(
            a1.clone() * a2.clone(),
            super::TypedValue::Number(1799182.0881075866)
        );
        assert_eq!(
            a1.clone() / a2.clone(),
            super::TypedValue::Number(2.2212569287310306)
        );
    }

    #[test]
    fn test_from_json_value() {
        use proto::common::common::DataTypeEnum;
        use std::collections::BTreeMap;
        let raw_data = "{\"key_1\": \"value_1\", \"key_2\": 1, \"key_3\": 3.14, \"key_4\": {\"sub_key_1\": \"subval_1\", \"sub_key_2\": 1, \"sub_key_3\": 3.14}, \"key_5\": [1,2,3,4], \"key_6\": [\"v1\", \"v2\", \"v3\"]}";

        let value = serde_json::from_str::<serde_json::Value>(raw_data);
        assert!(value.is_ok());

        let value = super::TypedValue::from_json_value(value.unwrap());
        assert_eq!(value.get_type(), DataTypeEnum::DATA_TYPE_ENUM_OBJECT);
        match value {
            super::TypedValue::Object(v) => {
                for index in 1..7 {
                    assert!(v.contains_key(&format!("key_{}", index)));
                }
                let val_1 = v.get(&format!("key_{}", 1)).unwrap();
                assert_eq!(val_1.get_type(), DataTypeEnum::DATA_TYPE_ENUM_STRING);
                match val_1 {
                    super::TypedValue::String(v) => assert_eq!(v.as_str(), "value_1"),
                    _ => panic!("unexpected type"),
                }

                let val_2 = v.get(&format!("key_{}", 2)).unwrap();
                assert_eq!(val_2.get_type(), DataTypeEnum::DATA_TYPE_ENUM_BIGINT);
                match val_2 {
                    super::TypedValue::BigInt(v) => assert_eq!(*v, 1),
                    _ => panic!("unexpected type"),
                }

                let val_3 = v.get(&format!("key_{}", 3)).unwrap();
                assert_eq!(val_3.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NUMBER);
                match val_3 {
                    super::TypedValue::Number(v) => assert_eq!(*v, 3.14),
                    _ => panic!("unexpected type"),
                }

                let val_4 = v.get(&format!("key_{}", 4)).unwrap();
                assert_eq!(val_4.get_type(), DataTypeEnum::DATA_TYPE_ENUM_OBJECT);
                let mut inner_obj = BTreeMap::new();
                inner_obj.insert(
                    "sub_key_1".to_string(),
                    super::TypedValue::String("subval_1".to_string()),
                );
                inner_obj.insert("sub_key_2".to_string(), super::TypedValue::BigInt(1));
                inner_obj.insert("sub_key_3".to_string(), super::TypedValue::Number(3.14));
                match val_4 {
                    super::TypedValue::Object(v) => assert_eq!(v, &inner_obj),
                    _ => panic!("unexpected type"),
                }

                let val_5 = v.get(&format!("key_{}", 5)).unwrap();
                assert_eq!(val_5.get_type(), DataTypeEnum::DATA_TYPE_ENUM_ARRAY);
                match val_5 {
                    super::TypedValue::Array(v) => assert_eq!(
                        v,
                        &(1..5)
                            .map(|index| super::TypedValue::BigInt(index))
                            .collect::<Vec<super::TypedValue>>()
                    ),
                    _ => panic!("unexpected type"),
                }

                let val_6 = v.get(&format!("key_{}", 6)).unwrap();
                assert_eq!(val_6.get_type(), DataTypeEnum::DATA_TYPE_ENUM_ARRAY);
                match val_6 {
                    super::TypedValue::Array(v) => assert_eq!(
                        v,
                        &(1..4)
                            .map(|index| super::TypedValue::String(format!("v{index}")))
                            .collect::<Vec<super::TypedValue>>()
                    ),
                    _ => panic!("unexpected type"),
                }
            }
            _ => panic!("unexpected type"),
        }
    }

    #[test]
    fn test_string_to_json_value() {
        let val = super::TypedValue::String("value".to_string());
        let value = val.to_json_value();

        assert!(value.is_string());
        assert!(value.as_str().is_some());
        assert_eq!(value.as_str().unwrap(), "value");
    }

    #[test]
    fn test_bigint_to_json_value() {
        let val = super::TypedValue::BigInt(198);
        let value = val.to_json_value();

        assert!(value.is_i64());
        assert!(value.as_i64().is_some());
        assert_eq!(value.as_i64().unwrap(), 198);
    }

    #[test]
    fn test_number_to_json_value() {
        let val = super::TypedValue::Number(198.198);
        let value = val.to_json_value();

        assert!(value.is_f64());
        assert!(value.as_f64().is_some());
        assert_eq!(value.as_f64().unwrap(), 198.198);
    }

    #[test]
    fn test_boolean_to_json_value() {
        let val = super::TypedValue::Boolean(false);
        let value = val.to_json_value();

        assert!(value.is_boolean());
        assert!(value.as_bool().is_some());
        assert_eq!(value.as_bool().unwrap(), false);

        let val = super::TypedValue::Boolean(true);
        let value = val.to_json_value();

        assert!(value.is_boolean());
        assert!(value.as_bool().is_some());
        assert_eq!(value.as_bool().unwrap(), true);
    }

    #[test]
    fn test_array_to_json_value() {
        {
            let val = super::TypedValue::Array(vec![
                super::TypedValue::BigInt(1),
                super::TypedValue::BigInt(2),
                super::TypedValue::BigInt(3),
            ]);
            let value = val.to_json_value();

            assert!(value.is_array());
            assert!(value.as_array().is_some());

            let arr = value.as_array().unwrap();

            let mut cursor = 1;
            for ele in arr {
                assert!(ele.is_i64());
                assert!(ele.as_i64().is_some());
                assert_eq!(ele.as_i64().unwrap(), cursor);
                cursor = cursor + 1;
            }
        }

        {
            let val = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
                super::TypedValue::String("v3".to_string()),
            ]);
            let value = val.to_json_value();

            assert!(value.is_array());
            assert!(value.as_array().is_some());

            let arr = value.as_array().unwrap();

            let mut cursor = 1;
            for ele in arr {
                assert!(ele.is_string());
                assert!(ele.as_str().is_some());
                assert_eq!(ele.as_str().unwrap(), format!("v{}", cursor).as_str());
                cursor = cursor + 1;
            }
        }
    }

    #[test]
    fn test_object_to_json_value() {
        use std::collections::BTreeMap;
        let mut obj = BTreeMap::default();
        obj.insert(
            "k1".to_string(),
            super::TypedValue::String("v1".to_string()),
        );
        obj.insert("k2".to_string(), super::TypedValue::BigInt(1));
        obj.insert("k3".to_string(), super::TypedValue::Number(2.0));

        let val = super::TypedValue::Object(obj);
        let value = val.to_json_value();

        assert!(value.is_object());
        assert!(value.as_object().is_some());

        let obj = value.as_object().unwrap();

        (1..4).for_each(|index| assert!(obj.contains_key(&format!("k{}", index))));
        let v1 = obj.get(&format!("k{}", 1)).unwrap();
        let v2 = obj.get(&format!("k{}", 2)).unwrap();
        let v3 = obj.get(&format!("k{}", 3)).unwrap();
        
        assert!(v1.is_string());
        assert_eq!(v1.as_str().unwrap(), "v1");

        assert!(v2.is_i64());
        assert_eq!(v2.as_i64().unwrap(), 1);

        assert!(v3.is_f64());
        assert_eq!(v3.as_f64().unwrap(), 2.0);

    }
}

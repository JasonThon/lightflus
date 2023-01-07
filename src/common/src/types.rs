use bytes::Buf;
use proto::common::Entry;
use proto::common::{DataTypeEnum, ResourceId};

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

/// TypedValue is the Rust mapping to the value in Javascript
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum TypedValue {
    // `string` in Javascript
    String(String),
    // `bigint` in Javascript
    BigInt(i64),
    // `boolean` in Javascript
    Boolean(bool),
    // `number` in Javascript
    Number(f64),
    // `null` in Javascript
    Null,
    // `object` in Javascript
    Object(BTreeMap<String, TypedValue>),
    // `array` in Javascript
    Array(Vec<TypedValue>),
    // `undefined` in Javascript
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

/// implementation of [`ToRedisArgs`] for [`TypedValue`]
/// Redis supports five data structures
/// - string
/// - hash
/// - list
/// - set
/// - zset
/// However, in redis, the commands of data in these data structures is distinct.
/// For now, Lightflus only support SET command, which means any value will be serialized as string and each key will be rewrite after call SET cmd
impl ToRedisArgs for TypedValue {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            TypedValue::String(v) => out.write_arg(v.as_bytes()),
            TypedValue::BigInt(v) => out.write_arg(&v.to_be_bytes()),
            TypedValue::Boolean(_) => out.write_arg(self.to_string().as_bytes()),
            TypedValue::Number(v) => out.write_arg(&v.to_be_bytes()),
            _ => out.write_arg(self.to_string().as_bytes()),
        }
    }
}

/// The result of to_string() of TypedValue must be the same as Typescript
impl ToString for TypedValue {
    fn to_string(&self) -> String {
        match self {
            TypedValue::String(v) => v.clone(),
            TypedValue::BigInt(v) => v.to_string(),
            TypedValue::Boolean(v) => v.to_string(),
            TypedValue::Number(v) => v.to_string(),
            TypedValue::Null => "null".to_string(),
            TypedValue::Object(v) => {
                let val = BTreeMap::from_iter(
                    v.iter()
                        .map(|pair| (pair.0.clone(), pair.1.to_json_value())),
                );
                serde_json::to_string(&val).unwrap_or_default()
            }
            TypedValue::Array(v) => serde_json::to_string(
                &v.iter()
                    .map(|val| val.to_json_value())
                    .collect::<Vec<serde_json::Value>>(),
            )
            .unwrap_or_default(),
            TypedValue::Invalid => "undefined".to_string(),
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
    /// get the slice of bytes with the head of data type symbol
    /// Example:
    /// ```
    /// use common::types::TypedValue;
    /// use proto::common::DataTypeEnum;
    ///
    /// fn main() {
    ///     let typed_string = TypedValue::String("1".to_string());
    ///     let data = typed_string.get_data();
    ///     assert_eq!(data[0], DataTypeEnum::String as u8)
    /// }
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
                .map_err(|err| tracing::error!("serialize object failed: {}", err))
                .unwrap_or_default(),
            TypedValue::Array(value) => {
                let result = Vec::from_iter(value.iter().map(|value| value.get_data()));
                serde_json::to_vec(&result)
                    .map_err(|err| tracing::error!("serialize array failed: {}", err))
                    .unwrap_or_default()
            }
            _ => vec![],
        };
        result.append(raw_data);

        result
    }

    /// To specify the type of value, we only accept the bytes with the head of data type symbol.
    /// It may be panic if the data has uncorrect head of data type symbol.
    /// Example:
    /// ```
    /// use common::types::TypedValue;
    /// use proto::common::DataTypeEnum;
    ///
    /// fn main() {
    ///     let typed_string = TypedValue::String("1".to_string());
    ///     let data = typed_string.get_data();
    ///     assert_eq!(data[0], DataTypeEnum::String as u8);
    ///     let typed_val = TypedValue::from_vec(&data);
    ///     assert_eq!(typed_val, typed_string)
    /// }
    pub fn from_vec(data: &Vec<u8>) -> Self {
        if data.is_empty() {
            return Self::Null;
        }
        let data_type = DataTypeEnum::from_i32(data[0] as i32).unwrap_or_default();

        match data_type {
            DataTypeEnum::String => match String::from_utf8(data[1..data.len()].to_vec()) {
                Ok(val) => TypedValue::String(val),
                Err(_) => TypedValue::Invalid,
            },
            DataTypeEnum::Bigint => {
                TypedValue::BigInt(data[1..data.len()].to_vec().as_slice().get_i64())
            }
            DataTypeEnum::Number => {
                TypedValue::Number(data[1..data.len()].to_vec().as_slice().get_f64())
            }
            DataTypeEnum::Boolean => TypedValue::Boolean(data[1] == 1),
            DataTypeEnum::Unspecified => TypedValue::Invalid,
            DataTypeEnum::Null => TypedValue::Null,
            DataTypeEnum::Object => TypedValue::Object(
                serde_json::from_slice::<BTreeMap<String, TypedValue>>(&data[1..data.len()])
                    .map_err(|err| tracing::error!("deserialize object failed: {}", err))
                    .unwrap_or_default(),
            ),
            DataTypeEnum::Array => {
                let val = serde_json::from_slice::<Vec<Vec<u8>>>(&data[1..data.len()])
                    .map_err(|err| tracing::error!("deserializ array failed:{}", err))
                    .unwrap_or_default();
                TypedValue::Array(Vec::from_iter(
                    val.iter().map(|data| TypedValue::from_vec(data)),
                ))
            }
        }
    }

    /// To specify the type of value, we only accept the bytes with the head of data type symbol.
    /// It may be panic if the data has uncorrect head of data type symbol.
    /// Example:
    /// ```
    /// use common::types::TypedValue;
    /// use proto::common::DataTypeEnum;
    ///
    /// fn main() {
    ///     let typed_string = TypedValue::String("1".to_string());
    ///     let data = typed_string.get_data();
    ///     assert_eq!(data[0], DataTypeEnum::String as u8);
    ///     let typed_val = TypedValue::from_slice(data.as_slice());
    ///     assert_eq!(typed_val, typed_string)
    /// }
    pub fn from_slice(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::Null;
        }
        let data_type = DataTypeEnum::from_i32(data[0] as i32).unwrap_or_default();

        match data_type {
            DataTypeEnum::String => match String::from_utf8(data[1..data.len()].to_vec()) {
                Ok(val) => TypedValue::String(val),
                Err(_) => TypedValue::Invalid,
            },
            DataTypeEnum::Bigint => {
                TypedValue::BigInt(data[1..data.len()].to_vec().as_slice().get_i64())
            }
            DataTypeEnum::Number => {
                TypedValue::Number(data[1..data.len()].to_vec().as_slice().get_f64())
            }
            DataTypeEnum::Boolean => TypedValue::Boolean(data[1] == 1),
            DataTypeEnum::Unspecified => TypedValue::Invalid,
            DataTypeEnum::Null => TypedValue::Null,
            DataTypeEnum::Object => TypedValue::Object(
                serde_json::from_slice::<BTreeMap<String, TypedValue>>(&data[1..data.len()])
                    .map_err(|err| tracing::error!("{err}"))
                    .unwrap_or(Default::default()),
            ),
            DataTypeEnum::Array => {
                let val = serde_json::from_slice::<Vec<Vec<u8>>>(&data[1..data.len()])
                    .map_err(|err| tracing::error!("{err}"))
                    .unwrap_or_default();
                TypedValue::Array(Vec::from_iter(
                    val.iter().map(|data| TypedValue::from_vec(data)),
                ))
            }
        }
    }

    /// get the data type of TypedValue
    pub fn get_type(&self) -> DataTypeEnum {
        match self {
            TypedValue::String(_) => DataTypeEnum::String,
            TypedValue::Number(_) => DataTypeEnum::Number,
            TypedValue::BigInt(_) => DataTypeEnum::Bigint,
            TypedValue::Null => DataTypeEnum::Null,
            TypedValue::Object(_) => DataTypeEnum::Object,
            TypedValue::Boolean(_) => DataTypeEnum::Boolean,
            TypedValue::Array(_) => DataTypeEnum::Array,
            _ => DataTypeEnum::Unspecified,
        }
    }

    pub fn from_slice_with_type(mut data: &[u8], data_type: DataTypeEnum) -> Self {
        if data.is_empty() {
            return if data_type != DataTypeEnum::Unspecified {
                Self::Null
            } else {
                Self::Invalid
            };
        }
        match data_type {
            DataTypeEnum::Unspecified => Self::Invalid,
            DataTypeEnum::Bigint => Self::BigInt(data.get_i64()),
            DataTypeEnum::Number => Self::Number(data.get_f64()),
            DataTypeEnum::Null => Self::Null,
            DataTypeEnum::String => Self::String(String::from_utf8_lossy(data).to_string()),
            DataTypeEnum::Boolean => Self::Boolean(data[0] == 1),
            DataTypeEnum::Object => {
                let value = serde_json::from_slice::<serde_json::Value>(data);
                match value {
                    Ok(val) => Self::from_json_value(val),
                    Err(err) => {
                        tracing::error!("deserialize json object failed: {}", err);
                        Self::Object(Default::default())
                    }
                }
            }
            DataTypeEnum::Array => {
                let value = serde_json::from_slice::<serde_json::Value>(data);
                match value {
                    Ok(val) => Self::from_json_value(val),
                    Err(err) => {
                        tracing::error!("deserialize json array failed: {}", err);
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
        data.extend_from_slice(&entry.value);
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

/// [`ResourceId`] generated by tonic does not implement [`Hash`]. We introduce a similar structure but with [`Hash`] derivable
#[derive(Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Debug)]
pub struct HashedResourceId {
    pub resource_id: String,
    pub namespace_id: String,
}

impl From<ResourceId> for HashedResourceId {
    fn from(id: ResourceId) -> Self {
        Self {
            resource_id: id.resource_id,
            namespace_id: id.namespace_id,
        }
    }
}

impl From<&ResourceId> for HashedResourceId {
    fn from(id: &ResourceId) -> Self {
        Self {
            resource_id: id.resource_id.clone(),
            namespace_id: id.namespace_id.clone(),
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

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::BTreeMap, str::FromStr};

    use bytes::Buf;

    #[test]
    pub fn test_typed_value_get_data() {
        use super::TypedValue;
        use bytes::Buf;
        use proto::common::DataTypeEnum;
        use std::collections::BTreeMap;

        let int = TypedValue::BigInt(1 << 30);
        let mut data = int.get_data();
        let _ = data.remove(0);
        assert_eq!(data.as_slice().get_i64(), 1 << 30);
        assert_eq!(int.get_type(), DataTypeEnum::Bigint);

        let double = super::TypedValue::Number(1.6546);
        let mut data = double.get_data();
        let _ = data.remove(0);
        assert_eq!(data.as_slice().get_f64(), 1.6546);
        assert_eq!(double.get_type(), DataTypeEnum::Number);

        let float = super::TypedValue::Null;
        assert_eq!(float.get_type(), DataTypeEnum::Null);
        let mut data = float.get_data();
        let _ = data.remove(0);
        assert_eq!(data.len(), 0);

        let string = super::TypedValue::String("test".to_string());
        assert_eq!(string.get_type(), DataTypeEnum::String);
        let mut data = string.get_data();
        let _ = data.remove(0);
        assert_eq!(String::from_utf8(data), Ok("test".to_string()));

        let boolean = super::TypedValue::Boolean(true);
        assert_eq!(boolean.get_type(), DataTypeEnum::Boolean);
        let mut data = boolean.get_data();
        let _ = data.remove(0);
        assert_eq!(data[0], 1);

        let boolean = super::TypedValue::Boolean(false);
        assert_eq!(boolean.get_type(), DataTypeEnum::Boolean);
        let mut data = boolean.get_data();
        let _ = data.remove(0);
        assert_eq!(data[0], 0);

        let arr = super::TypedValue::Array(vec![
            super::TypedValue::Number(1.0),
            super::TypedValue::Number(2.0),
            super::TypedValue::Number(3.0),
        ]);
        assert_eq!(arr.get_type(), DataTypeEnum::Array);
        let mut data = arr.get_data();
        let _ = data.remove(0);
        let data = serde_json::from_slice::<Vec<Vec<u8>>>(data.as_slice());
        assert!(data.is_ok());
        let data = data.unwrap();
        assert_eq!(data.len(), 3);
        let mut index = 1.0;
        data.iter().for_each(|entry| {
            let val = super::TypedValue::from_vec(entry);
            assert_eq!(val.get_type(), DataTypeEnum::Number);
            match val {
                super::TypedValue::Number(v) => assert_eq!(v, index),
                _ => panic!("unexpected type"),
            };

            index = index + 1.0;
        });

        let mut obj = BTreeMap::new();
        obj.insert(
            "k1".to_string(),
            super::TypedValue::String("v1".to_string()),
        );
        obj.insert("k2".to_string(), super::TypedValue::BigInt(1));
        obj.insert("k3".to_string(), super::TypedValue::Number(1.0));
        let obj = super::TypedValue::Object(obj);
        assert_eq!(obj.get_type(), DataTypeEnum::Object);
        let mut data = obj.get_data();
        let _ = data.remove(0);

        let val = serde_json::from_slice::<BTreeMap<String, super::TypedValue>>(data.as_slice());
        assert!(val.is_ok());

        let val = val.expect("");

        (1..4).for_each(|index| {
            let key = &format!("k{}", index);
            assert!(val.contains_key(key));
            let value = val.get(key).unwrap();

            if key == "k1" {
                assert_eq!(value.clone(), super::TypedValue::String("v1".to_string()));
            }

            if key == "k2" {
                assert_eq!(value.clone(), super::TypedValue::BigInt(1));
            }

            if key == "k3" {
                assert_eq!(value.clone(), super::TypedValue::Number(1.0));
            }
        })
    }

    #[test]
    pub fn test_typed_value_left_int_dual_op() {
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
    pub fn test_typed_value_left_long_dual_op() {
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
    pub fn test_typed_value_left_float_dual_op() {
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
    pub fn test_typed_value_partial_order() {
        {
            let a1 = super::TypedValue::String("v1".to_string());
            let a2 = super::TypedValue::String("v1".to_string());
            assert_eq!(a1, a2);
        }

        {
            let a1 = super::TypedValue::Number(1.0);
            let a2 = super::TypedValue::Number(1.0);
            assert_eq!(a1, a2);
            assert!(a1 == a2);

            let a1 = super::TypedValue::Number(1.0);
            let a2 = super::TypedValue::Number(2.0);
            assert_ne!(a1, a2);
            assert!(a1 < a2);

            let a1 = super::TypedValue::Number(2.0);
            let a2 = super::TypedValue::Number(1.0);
            assert_ne!(a1, a2);
            assert!(a1 > a2);

            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::Number(1.0);
            assert_ne!(a1, a2);
            assert!(a1 > a2);

            let a1 = super::TypedValue::Number(2.0);
            let a2 = super::TypedValue::BigInt(1);
            assert_ne!(a1, a2);
            assert!(a1 > a2);

            let a1 = super::TypedValue::BigInt(1);
            let a2 = super::TypedValue::Number(1.5);
            assert_ne!(a1, a2);
            assert!(a1 < a2);
        }

        {
            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::BigInt(2);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::BigInt(1);
            assert_ne!(a1, a2);
            assert!(a1 > a2);

            let a1 = super::TypedValue::BigInt(1);
            let a2 = super::TypedValue::BigInt(2);
            assert_ne!(a1, a2);
            assert!(a1 < a2);
        }

        {
            let a1 = super::TypedValue::Boolean(true);
            let a2 = super::TypedValue::Boolean(true);
            assert_eq!(a1, a2);
            assert!(a1 == a2);

            let a1 = super::TypedValue::Boolean(true);
            let a2 = super::TypedValue::Boolean(false);
            assert_ne!(a1, a2);
            assert!(a1 > a2);

            let a1 = super::TypedValue::Boolean(false);
            let a2 = super::TypedValue::Boolean(true);
            assert_ne!(a1, a2);
            assert!(a1 < a2);
        }

        {
            let a1 = super::TypedValue::Null;
            let a2 = super::TypedValue::Boolean(true);
            assert_ne!(a1, a2);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, None);

            let a1 = super::TypedValue::Null;
            let a2 = super::TypedValue::Null;
            assert_eq!(a1, a2);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Null;
            let a2 = super::TypedValue::Invalid;
            assert_eq!(a1, a2);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Invalid;
            let a2 = super::TypedValue::Null;
            assert_eq!(a1, a2);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Invalid;
            let a2 = super::TypedValue::Invalid;
            assert_eq!(a1, a2);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));
        }

        {
            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::String("v2".to_string()),
                super::TypedValue::String("v3".to_string()),
            ]);

            assert!(a1 < a2);
            assert!(a2 > a1);

            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::Number(1.0),
                super::TypedValue::Number(2.0),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::Number(1.0),
                super::TypedValue::Number(2.0),
            ]);
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::Number(1.0),
                super::TypedValue::Number(2.0),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::Number(2.0),
                super::TypedValue::Number(3.0),
            ]);

            assert!(a1 < a2);
            assert!(a2 > a1);
        }

        {
            let a1 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            let a2 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            let order = a1.partial_cmp(&a2);
            assert_eq!(order, Some(Ordering::Equal));

            let a1 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            let a2 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                    (
                        "k3".to_string(),
                        super::TypedValue::String("v3".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));

            assert!(a1 < a2);
            assert!(a2 > a1);
        }
    }

    #[test]
    pub fn test_typed_value_partial_eq() {
        {
            let a1 = super::TypedValue::Boolean(true);
            let a2 = super::TypedValue::Boolean(true);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::Boolean(true);
            let a2 = super::TypedValue::Boolean(false);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::Boolean(false);
            let a2 = super::TypedValue::Boolean(true);
            assert_ne!(a1, a2);
        }

        {
            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::BigInt(2);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::BigInt(1);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::BigInt(1);
            let a2 = super::TypedValue::BigInt(2);
            assert_ne!(a1, a2);
        }

        {
            let a1 = super::TypedValue::Number(1.0);
            let a2 = super::TypedValue::Number(1.0);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::Number(1.0);
            let a2 = super::TypedValue::Number(2.0);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::Number(2.0);
            let a2 = super::TypedValue::Number(1.0);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::BigInt(2);
            let a2 = super::TypedValue::Number(1.0);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::Number(2.0);
            let a2 = super::TypedValue::BigInt(1);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::BigInt(1);
            let a2 = super::TypedValue::Number(1.5);
            assert_ne!(a1, a2);

            let a1 = super::TypedValue::BigInt(1);
            let a2 = super::TypedValue::Number(1.0);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::Number(1.0);
            let a2 = super::TypedValue::BigInt(1);
            assert_eq!(a1, a2);
        }

        {
            let a1 = super::TypedValue::String("v1".to_string());
            let a2 = super::TypedValue::String("v1".to_string());
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::String("v1".to_string());
            let a2 = super::TypedValue::String("v2".to_string());
            assert_ne!(a1, a2);
        }

        {
            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
            ]);
            let a2 = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v3".to_string()),
            ]);
            assert_ne!(a1, a2);
        }

        {
            let a1 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            let a2 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            assert_eq!(a1, a2);

            let a1 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            let a2 = super::TypedValue::Object(BTreeMap::from_iter(
                [
                    (
                        "k1".to_string(),
                        super::TypedValue::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        super::TypedValue::String("v2".to_string()),
                    ),
                    (
                        "k3".to_string(),
                        super::TypedValue::String("v3".to_string()),
                    ),
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone())),
            ));
            assert_ne!(a1, a2);
        }
    }

    #[test]
    fn test_typed_value_default() {
        let val: super::TypedValue = Default::default();

        assert_eq!(val, super::TypedValue::Invalid);
    }

    #[test]
    pub fn test_from_json_value() {
        use proto::common::DataTypeEnum;
        use std::collections::BTreeMap;
        let raw_data = "{\"key_1\": \"value_1\", \"key_2\": 1, \"key_3\": 3.14, \"key_4\": {\"sub_key_1\": \"subval_1\", \"sub_key_2\": 1, \"sub_key_3\": 3.14}, \"key_5\": [1,2,3,4], \"key_6\": [\"v1\", \"v2\", \"v3\"],\"key_7\":true,\"key_8\":null}";

        let value = serde_json::from_str::<serde_json::Value>(raw_data);
        assert!(value.is_ok());

        let value = super::TypedValue::from_json_value(value.unwrap());
        assert_eq!(value.get_type(), DataTypeEnum::Object);
        match value {
            super::TypedValue::Object(v) => {
                for index in 1..7 {
                    assert!(v.contains_key(&format!("key_{}", index)));
                }
                let val_1 = v.get(&format!("key_{}", 1)).unwrap();
                assert_eq!(val_1.get_type(), DataTypeEnum::String);
                match val_1 {
                    super::TypedValue::String(v) => assert_eq!(v.as_str(), "value_1"),
                    _ => panic!("unexpected type"),
                }

                let val_2 = v.get(&format!("key_{}", 2)).unwrap();
                assert_eq!(val_2.get_type(), DataTypeEnum::Bigint);
                match val_2 {
                    super::TypedValue::BigInt(v) => assert_eq!(*v, 1),
                    _ => panic!("unexpected type"),
                }

                let val_3 = v.get(&format!("key_{}", 3)).unwrap();
                assert_eq!(val_3.get_type(), DataTypeEnum::Number);
                match val_3 {
                    super::TypedValue::Number(v) => assert_eq!(*v, 3.14),
                    _ => panic!("unexpected type"),
                }

                let val_4 = v.get(&format!("key_{}", 4)).unwrap();
                assert_eq!(val_4.get_type(), DataTypeEnum::Object);
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
                assert_eq!(val_5.get_type(), DataTypeEnum::Array);
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
                assert_eq!(val_6.get_type(), DataTypeEnum::Array);
                match val_6 {
                    super::TypedValue::Array(v) => assert_eq!(
                        v,
                        &(1..4)
                            .map(|index| super::TypedValue::String(format!("v{index}")))
                            .collect::<Vec<super::TypedValue>>()
                    ),
                    _ => panic!("unexpected type"),
                }

                let val_7 = v.get(&format!("key_{}", 7)).unwrap();
                assert_eq!(val_7.get_type(), DataTypeEnum::Boolean);
                match val_7 {
                    super::TypedValue::Boolean(v) => assert_eq!(v, &true),
                    _ => panic!("unexpected type"),
                }

                let val_8 = v.get(&format!("key_{}", 8)).unwrap();
                assert_eq!(val_8.get_type(), DataTypeEnum::Null);
            }
            _ => panic!("unexpected type"),
        }
    }

    #[test]
    pub fn test_string_to_json_value() {
        let val = super::TypedValue::String("value".to_string());
        assert_eq!(
            val.to_json_value(),
            serde_json::Value::String("value".to_string())
        );
    }

    #[test]
    fn test_bigint_to_json_value() {
        let val = super::TypedValue::BigInt(198);
        assert_eq!(
            val.to_json_value(),
            serde_json::Value::Number(serde_json::Number::from_str("198").unwrap())
        );
    }

    #[test]
    fn test_number_to_json_value() {
        let val = super::TypedValue::Number(198.198);
        assert_eq!(
            val.to_json_value(),
            serde_json::Value::Number(serde_json::Number::from_str("198.198").unwrap())
        );
    }

    #[test]
    fn test_boolean_to_json_value() {
        let val = super::TypedValue::Boolean(false);
        assert_eq!(val.to_json_value(), serde_json::Value::Bool(false));

        let val = super::TypedValue::Boolean(true);
        assert_eq!(val.to_json_value(), serde_json::Value::Bool(true));
    }

    #[test]
    fn test_array_to_json_value() {
        {
            let val = super::TypedValue::Array(vec![
                super::TypedValue::BigInt(1),
                super::TypedValue::BigInt(2),
                super::TypedValue::BigInt(3),
            ]);

            assert_eq!(
                val.to_json_value(),
                serde_json::Value::Array(vec![
                    serde_json::Value::Number(serde_json::Number::from_str("1").unwrap(),),
                    serde_json::Value::Number(serde_json::Number::from_str("2").unwrap(),),
                    serde_json::Value::Number(serde_json::Number::from_str("3").unwrap(),)
                ])
            );
        }

        {
            let val = super::TypedValue::Array(vec![
                super::TypedValue::String("v1".to_string()),
                super::TypedValue::String("v2".to_string()),
                super::TypedValue::String("v3".to_string()),
            ]);

            assert_eq!(
                val.to_json_value(),
                serde_json::Value::Array(vec![
                    serde_json::Value::String("v1".to_string()),
                    serde_json::Value::String("v2".to_string()),
                    serde_json::Value::String("v3".to_string()),
                ])
            );
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

        assert_eq!(
            val.to_json_value(),
            serde_json::Value::Object(serde_json::Map::from_iter(
                [
                    (
                        "k1".to_string(),
                        serde_json::Value::String("v1".to_string()),
                    ),
                    (
                        "k2".to_string(),
                        serde_json::Value::Number(serde_json::Number::from_str("1").unwrap(),)
                    ),
                    (
                        "k3".to_string(),
                        serde_json::Value::Number(serde_json::Number::from_str("2.0").unwrap(),)
                    )
                ]
                .iter()
                .map(|entry| (entry.0.clone(), entry.1.clone()))
            ))
        );
    }

    #[test]
    fn test_typed_value_to_redis_args() {
        use redis::ToRedisArgs;

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::String("value".to_string());
            val.write_redis_args(&mut writer);

            assert_eq!(writer.len(), 1);

            let val = &writer[0];

            assert_eq!(String::from_utf8(val.clone()), Ok("value".to_string()))
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::BigInt(10);
            val.write_redis_args(&mut writer);

            assert_eq!(writer.len(), 1);

            let val = &writer[0];

            assert_eq!(val.as_slice().get_i64(), 10)
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Number(10.78);
            val.write_redis_args(&mut writer);

            assert_eq!(writer.len(), 1);

            let val = &writer[0];

            assert_eq!(val.as_slice().get_f64(), 10.78)
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Null;
            val.write_redis_args(&mut writer);

            assert_eq!(writer.len(), 1);

            let val = &writer[0];

            assert_eq!(String::from_utf8(val.clone()), Ok("null".to_string()))
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Invalid;
            val.write_redis_args(&mut writer);

            assert_eq!(writer.len(), 1);
            let val = &writer[0];
            assert_eq!(String::from_utf8(val.clone()), Ok("undefined".to_string()))
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Array(vec![
                super::TypedValue::BigInt(1),
                super::TypedValue::BigInt(2),
                super::TypedValue::BigInt(3),
            ]);
            val.write_redis_args(&mut writer);
            assert_eq!(writer.len(), 1);

            let val = &writer[0];
            assert_eq!(String::from_utf8(val.clone()), Ok("[1,2,3]".to_string()))
        }

        {
            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Boolean(true);
            val.write_redis_args(&mut writer);
            assert_eq!(writer.len(), 1);

            let val = &writer[0];
            assert_eq!(String::from_utf8(val.clone()), Ok("true".to_string()));

            let mut writer: Vec<Vec<u8>> = vec![];
            let val = super::TypedValue::Boolean(false);
            val.write_redis_args(&mut writer);
            assert_eq!(writer.len(), 1);

            let val = &writer[0];
            assert_eq!(String::from_utf8(val.clone()), Ok("false".to_string()))
        }
    }

    #[test]
    fn test_from_slice_with_type() {
        use proto::common::DataTypeEnum;

        {
            let val =
                super::TypedValue::from_slice_with_type("v1".as_bytes(), DataTypeEnum::String);
            assert_eq!(val.get_type(), DataTypeEnum::String);
            assert_eq!(val, super::TypedValue::String("v1".to_string()));
        }

        {
            let val: i64 = 1;
            let val = val.to_be_bytes();
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Bigint);
            assert_eq!(val.get_type(), DataTypeEnum::Bigint);
            assert_eq!(val, super::TypedValue::BigInt(1));
        }

        {
            let val: f64 = 1.0;
            let val = val.to_be_bytes();
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Number);
            assert_eq!(val.get_type(), DataTypeEnum::Number);
            assert_eq!(val, super::TypedValue::Number(1.0));
        }

        {
            let val = [1];
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Boolean);
            assert_eq!(val.get_type(), DataTypeEnum::Boolean);
            assert_eq!(val, super::TypedValue::Boolean(true));

            let val = [0];
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Boolean);
            assert_eq!(val.get_type(), DataTypeEnum::Boolean);
            assert_eq!(val, super::TypedValue::Boolean(false));
        }

        {
            let val = [1];
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Null);
            assert_eq!(val.get_type(), DataTypeEnum::Null);
            assert_eq!(val, super::TypedValue::Null);
        }

        {
            let val = [];
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Unspecified);
            assert_eq!(val.get_type(), DataTypeEnum::Unspecified);
            assert_eq!(val, super::TypedValue::Invalid);

            let val = [];
            let val = super::TypedValue::from_slice_with_type(&val, DataTypeEnum::Boolean);
            assert_eq!(val.get_type(), DataTypeEnum::Null);
            assert_eq!(val, super::TypedValue::Null);
        }

        {
            let raw_data = "{\"key_1\": \"value_1\", \"key_2\": 1, \"key_3\": 3.14, \"key_4\": {\"sub_key_1\": \"subval_1\", \"sub_key_2\": 1, \"sub_key_3\": 3.14}, \"key_5\": [1,2,3,4], \"key_6\": [\"v1\", \"v2\", \"v3\"],\"key_7\":true,\"key_8\":null}";
            let val =
                super::TypedValue::from_slice_with_type(raw_data.as_bytes(), DataTypeEnum::Object);
            assert_eq!(val.get_type(), DataTypeEnum::Object);

            let obj = BTreeMap::from_iter(
                [
                    ("key_1", super::TypedValue::String("value_1".to_string())),
                    ("key_2", super::TypedValue::BigInt(1)),
                    ("key_3", super::TypedValue::Number(3.14)),
                    (
                        "key_4",
                        super::TypedValue::Object(BTreeMap::from_iter(
                            [
                                (
                                    "sub_key_1",
                                    super::TypedValue::String("subval_1".to_string()),
                                ),
                                ("sub_key_2", super::TypedValue::BigInt(1)),
                                ("sub_key_3", super::TypedValue::Number(3.14)),
                            ]
                            .iter()
                            .map(|entry| (entry.0.to_string(), entry.1.clone())),
                        )),
                    ),
                    (
                        "key_5",
                        super::TypedValue::Array(vec![
                            super::TypedValue::BigInt(1),
                            super::TypedValue::BigInt(2),
                            super::TypedValue::BigInt(3),
                            super::TypedValue::BigInt(4),
                        ]),
                    ),
                    (
                        "key_6",
                        super::TypedValue::Array(vec![
                            super::TypedValue::String("v1".to_string()),
                            super::TypedValue::String("v2".to_string()),
                            super::TypedValue::String("v3".to_string()),
                        ]),
                    ),
                    ("key_7", super::TypedValue::Boolean(true)),
                    ("key_8", super::TypedValue::Null),
                ]
                .iter()
                .map(|entry| (entry.0.to_string(), entry.1.clone())),
            );

            assert_eq!(val, super::TypedValue::Object(obj));
        }
    }
}

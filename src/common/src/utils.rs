use crate::{
    net::hostname,
    types::{
        BIGINT_SYMBOL, BOOLEAN_SYMBOL, NULL_SYMBOL, NUMBER_SYMBOL, OBJECT_SYMBOL, STRING_SYMBOL,
        UNDEFINED_SYMBOL,
    },
};
use bytes::BytesMut;
use prost::{DecodeError, Message};
use proto::common::{DataTypeEnum, Dataflow, DataflowMeta, OperatorInfo, ResourceId};
use serde::de::Error;
use std::collections::HashMap;
use std::env;
use std::io::Read;

pub struct Args {
    args: HashMap<String, Arg>,
}

impl Default for Args {
    fn default() -> Self {
        let mut current_arg = Arg {
            key: "".to_string(),
            value: "".to_string(),
        };
        let mut map = HashMap::new();

        env::args().for_each(|arg| Self::process_arg(arg, &mut current_arg, &mut map));

        Self { args: map.clone() }
    }
}

impl Args {
    pub fn arg(&self, flag: &str) -> Option<Arg> {
        let key = flag.to_string();
        self.args.get(&key).map(|val| val.clone())
    }

    pub(crate) fn process_arg(
        arg: String,
        mut current_arg: &mut Arg,
        map: &mut HashMap<String, Arg>,
    ) {
        let is_flag = arg.starts_with("-");
        if is_flag {
            if !current_arg.key.is_empty() {
                panic!(
                    "flag {} has invalid value {}",
                    current_arg.key.as_str(),
                    arg
                );
            }
            if !current_arg.is_empty() {
                current_arg.clear();
            }
            current_arg.key = arg[1..arg.len()].to_string();
        } else {
            current_arg.value = arg.clone();
            let _ = map.insert(current_arg.key.clone(), current_arg.clone());
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Arg {
    pub key: String,
    pub value: String,
}

impl Arg {
    pub(crate) fn is_empty(&self) -> bool {
        self.key.is_empty() && self.value.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.key = "".to_string();
        self.value = "".to_string()
    }
}

pub fn get_env(k: &str) -> Option<String> {
    match env::var(k.to_string()) {
        Ok(var) => Some(var),
        Err(err) => {
            tracing::error!("{}", err);
            None
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub fn from_reader<R: std::io::Read>(reader: R) -> serde_json::Result<String> {
    let ref mut buf = Default::default();
    let mut buf_reader = std::io::BufReader::new(reader);
    buf_reader
        .read_to_string(buf)
        .map_err(|err| serde_json::Error::custom(format!("fail to read from reader: {}", err)))
        .map(|_| replace_by_env(buf))
}

pub fn from_str(value: &str) -> String {
    replace_by_env(value)
}

fn replace_by_env(value: &str) -> String {
    let ref mut buf = value.to_string();
    let reg = regex::Regex::new("\\$\\{[^}]+\\}").unwrap();
    reg.captures_iter(value).for_each(|captures| {
        captures.iter().for_each(|matched| match matched {
            Some(m) => match std::env::var(m.as_str()[2..(m.end() - m.start() - 1)].to_string()) {
                Ok(var) => {
                    let result = buf.replace(m.as_str(), var.as_str());
                    buf.clear();
                    buf.insert_str(0, result.as_str())
                }
                Err(_) => {}
            },
            _ => {}
        })
    });
    buf.clone()
}

pub fn is_remote_operator(operator: &OperatorInfo) -> bool {
    let host_addr = &operator.host_addr.as_ref();
    if host_addr.is_none() || host_addr.filter(|addr| &addr.host == "localhost").is_some() {
        return false;
    }

    hostname()
        .map(|host| {
            operator.host_addr.is_some() && host_addr.filter(|addr| addr.host != host).is_some()
        })
        .unwrap_or(false)
}

pub fn to_dataflow(
    job_id: &ResourceId,
    operators: &Vec<OperatorInfo>,
    meta: &[DataflowMeta],
) -> Dataflow {
    let mut dataflow = Dataflow::default();

    dataflow.job_id = Some(job_id.clone());
    dataflow.nodes = operators
        .iter()
        .map(|entry| (entry.operator_id, (*entry).clone()))
        .collect();

    dataflow.meta = meta
        .iter()
        .filter(|elem| dataflow.nodes.contains_key(&elem.center))
        .map(|elem| elem.clone())
        .collect();

    dataflow
}

pub fn from_type_symbol(symbol: String) -> DataTypeEnum {
    let raw = symbol.as_str();
    if raw == STRING_SYMBOL {
        DataTypeEnum::String
    } else if raw == NUMBER_SYMBOL {
        DataTypeEnum::Number
    } else if raw == OBJECT_SYMBOL {
        DataTypeEnum::Object
    } else if raw == BOOLEAN_SYMBOL {
        DataTypeEnum::Boolean
    } else if raw == BIGINT_SYMBOL {
        DataTypeEnum::Bigint
    } else if raw == NULL_SYMBOL {
        DataTypeEnum::Null
    } else if raw == UNDEFINED_SYMBOL {
        DataTypeEnum::Unspecified
    } else {
        DataTypeEnum::Unspecified
    }
}

pub fn pb_to_bytes_mut<T: Message>(message: T) -> BytesMut {
    let mut bytes = BytesMut::new();
    bytes.extend_from_slice(&message.encode_to_vec());

    bytes
}

#[cfg(not(tarpaulin_include))]
pub fn from_pb_slice<T: Message + std::default::Default>(data: &[u8]) -> Result<T, DecodeError> {
    prost::Message::decode(data)
}

#[cfg(not(tarpaulin_include))]
pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use proto::{
        common::{operator_info::Details, ResourceId},
        common_impl::DataflowValidateError,
    };

    #[test]
    fn test_process_arg_success() {
        use std::collections::HashMap;
        let mut current_arg = Default::default();
        let mut map = HashMap::new();
        super::Args::process_arg("-f".to_string(), &mut current_arg, &mut map);
        assert!(!map.contains_key(&"f".to_string()));
        assert_eq!(current_arg.key, "f".to_string());
        assert_eq!(current_arg.value, "".to_string());

        super::Args::process_arg("src".to_string(), &mut current_arg, &mut map);
        assert!(map.contains_key(&"f".to_string()));
        assert_eq!(current_arg.key, "f".to_string());
        assert_eq!(current_arg.value, "src".to_string());

        assert_eq!(
            map.get(&"f".to_string()),
            Some(&super::Arg {
                key: "f".to_string(),
                value: "src".to_string()
            })
        )
    }

    #[test]
    #[should_panic(expected = "flag f has invalid value -s")]
    fn test_process_arg_failed() {
        use std::collections::HashMap;
        let mut current_arg = Default::default();
        let mut map = HashMap::new();
        super::Args::process_arg("-f".to_string(), &mut current_arg, &mut map);
        assert!(!map.contains_key(&"f".to_string()));
        assert_eq!(current_arg.key, "f".to_string());
        assert_eq!(current_arg.value, "".to_string());

        super::Args::process_arg("-s".to_string(), &mut current_arg, &mut map);
    }

    #[test]
    fn test_from_type_symbol() {
        use proto::common::DataTypeEnum;
        assert_eq!(
            super::from_type_symbol(super::STRING_SYMBOL.to_string()),
            DataTypeEnum::String
        );

        assert_eq!(
            super::from_type_symbol(super::NUMBER_SYMBOL.to_string()),
            DataTypeEnum::Number
        );

        assert_eq!(
            super::from_type_symbol(super::OBJECT_SYMBOL.to_string()),
            DataTypeEnum::Object
        );

        assert_eq!(
            super::from_type_symbol(super::BOOLEAN_SYMBOL.to_string()),
            DataTypeEnum::Boolean
        );

        assert_eq!(
            super::from_type_symbol(super::BIGINT_SYMBOL.to_string()),
            DataTypeEnum::Bigint
        );

        assert_eq!(
            super::from_type_symbol(super::NULL_SYMBOL.to_string()),
            DataTypeEnum::Null
        );

        assert_eq!(
            super::from_type_symbol(super::UNDEFINED_SYMBOL.to_string()),
            DataTypeEnum::Unspecified
        );

        assert_eq!(
            super::from_type_symbol("sss".to_string()),
            DataTypeEnum::Unspecified
        );
    }

    #[test]
    fn test_to_dataflow() {
        use proto::common::DataflowMeta;
        use proto::common::OperatorInfo;
        use proto::common::ResourceId;
        use std::collections::HashMap;

        let job_id = ResourceId::default();
        let mut op1 = OperatorInfo::default();
        op1.operator_id = 0;
        let mut op2 = OperatorInfo::default();
        op2.operator_id = 1;
        let operators = vec![op1.clone(), op2.clone()];

        let meta1 = DataflowMeta {
            center: 0,
            neighbors: vec![1],
        };

        let dataflow = super::to_dataflow(&job_id, &operators, &[meta1.clone()]);

        let mut nodes = HashMap::default();
        nodes.insert(0, op1);
        nodes.insert(1, op2);

        assert_eq!(dataflow.job_id, Some(job_id));
        assert_eq!(dataflow.meta, &[meta1]);
        assert_eq!(dataflow.nodes, nodes);
    }

    #[test]
    fn test_validate_dataflow_success() {
        use proto::common::Dataflow;
        use proto::common::DataflowMeta;
        use proto::common::OperatorInfo;
        use std::collections::HashMap;

        let mut dataflow = Dataflow::default();
        dataflow.job_id = Some(ResourceId {
            resource_id: "resourceId".to_string(),
            namespace_id: "namespace_id".to_string(),
        });
        let mut meta = DataflowMeta::default();
        meta.center = 0;
        meta.neighbors = vec![1, 2, 3];

        dataflow.meta = vec![meta];

        let mut nodes = HashMap::default();
        let mut info_1 = OperatorInfo::default();
        info_1.operator_id = 0;
        info_1.details = Some(Details::Filter(Default::default()));
        nodes.insert(0, info_1);
        dataflow.nodes = nodes.clone();

        let result = dataflow.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            DataflowValidateError::OperatorInfoMissing(_) => {}
            _ => panic!("unexpected error"),
        };

        (1..4).for_each(|index| {
            let mut info = OperatorInfo::default();
            info.operator_id = index;
            info.details = Some(Details::Filter(Default::default()));
            nodes.insert(index, info);
        });

        dataflow.nodes = nodes;
        let result = dataflow.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataflow_is_cyclic() {
        use proto::common::Dataflow;
        use proto::common::DataflowMeta;
        use proto::common::OperatorInfo;
        use std::collections::HashMap;

        let mut dataflow = Dataflow::default();
        let mut meta = DataflowMeta::default();
        meta.center = 0;
        meta.neighbors = vec![1, 2, 3];
        let mut meta_1 = DataflowMeta::default();
        meta_1.center = 1;
        meta_1.neighbors = vec![2, 3, 4];
        let mut meta_2 = DataflowMeta::default();
        meta_2.center = 4;
        meta_2.neighbors = vec![0];

        dataflow.meta = vec![meta, meta_1, meta_2];
        dataflow.job_id = Some(ResourceId {
            resource_id: "resourceId".to_string(),
            namespace_id: "namespace_id".to_string(),
        });
        let mut nodes = HashMap::default();

        (0..5).for_each(|index| {
            let mut info = OperatorInfo::default();
            info.operator_id = index;
            info.details = Some(Details::Filter(Default::default()));
            nodes.insert(index, info);
        });

        dataflow.nodes = nodes;

        let result = dataflow.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            DataflowValidateError::CyclicDataflow => {}
            _ => panic!("unexpected error"),
        };
    }

    #[test]
    fn test_serde_env() {
        let origin = "{\"name\":\"${your.name}\", \"card\": \"${your.card}\"}";
        std::env::set_var("your.name", "jason");
        std::env::set_var("your.card", "111");
        let target = common::utils::from_str(origin);
        let result = serde_json::from_str::<Name>(target.as_str());
        if result.is_err() {
            print!("{:?}", result.as_ref().unwrap_err())
        }
        assert!(result.is_ok());
        let name = result.unwrap();
        assert_eq!(&name.name, &"jason".to_string());
        assert_eq!(&name.card, &"111".to_string());
    }

    #[test]
    fn test_env_var_get() {
        std::env::set_var("your.name", "jason");
        let result = std::env::var("your.name".to_string());
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), &"jason".to_string())
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Name {
        name: String,
        card: String,
    }
}

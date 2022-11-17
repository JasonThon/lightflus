use crate::{
    err::DataflowValidateError,
    net::hostname,
    types::{
        BIGINT_SYMBOL, BOOLEAN_SYMBOL, NULL_SYMBOL, NUMBER_SYMBOL, OBJECT_SYMBOL, STRING_SYMBOL,
        UNDEFINED_SYMBOL,
    },
};
use bytes::BytesMut;
use proto::common::{
    common::{DataTypeEnum, ResourceId},
    stream::{Dataflow, DataflowMeta, OperatorInfo},
};
use protobuf::Message;
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
            log::error!("{}", err);
            None
        }
    }
}

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
    if operator.get_host_addr().get_host() == "localhost" || !operator.has_host_addr() {
        return false;
    }

    hostname()
        .map(|host| operator.get_host_addr().host != host)
        .unwrap_or(false)
}

pub fn to_dataflow(
    job_id: &ResourceId,
    operators: &Vec<OperatorInfo>,
    meta: &[DataflowMeta],
) -> Dataflow {
    let mut dataflow = Dataflow::new();
    dataflow.set_job_id(job_id.clone());
    dataflow.set_nodes(
        operators
            .iter()
            .map(|entry| (entry.get_operator_id(), (*entry).clone()))
            .collect(),
    );
    dataflow.set_meta(
        meta.iter()
            .filter(|elem| dataflow.get_nodes().contains_key(&elem.center))
            .map(|elem| elem.clone())
            .collect(),
    );

    dataflow
}

pub fn validate_dataflow(dataflow: &Dataflow) -> Result<(), DataflowValidateError> {
    let mut metas = dataflow.get_meta().to_vec();
    metas.sort_by(|prev, next| prev.center.cmp(&next.center));

    for meta in &metas {
        if !dataflow.get_nodes().contains_key(&meta.center) {
            return Err(DataflowValidateError::OperatorInfoMissing(format!(
                "operatorInfo of node {} is missing",
                meta.center
            )));
        }

        for neighbor in meta.get_neighbors() {
            if neighbor < &meta.center {
                return Err(DataflowValidateError::CyclicDataflow);
            }

            if !dataflow.get_nodes().contains_key(neighbor) {
                return Err(DataflowValidateError::OperatorInfoMissing(format!(
                    "operatorInfo of node {} is missing",
                    neighbor
                )));
            }
        }
    }

    return Ok(());
}

pub fn from_type_symbol(symbol: String) -> DataTypeEnum {
    let raw = symbol.as_str();
    if raw == STRING_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_STRING
    } else if raw == NUMBER_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_NUMBER
    } else if raw == OBJECT_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_OBJECT
    } else if raw == BOOLEAN_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN
    } else if raw == BIGINT_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_BIGINT
    } else if raw == NULL_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_NULL
    } else if raw == UNDEFINED_SYMBOL {
        DataTypeEnum::DATA_TYPE_ENUM_NULL
    } else {
        DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED
    }
}

pub fn pb_to_bytes_mut<T: Message>(message: T) -> BytesMut {
    let ref mut raw_data = vec![];
    let mut bytes = BytesMut::new();
    if message.write_to_vec(raw_data).is_ok() {
        bytes.extend_from_slice(raw_data);
    }

    bytes
}

#[cfg(test)]
mod tests {

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
        use proto::common::common::DataTypeEnum;
        assert_eq!(
            super::from_type_symbol(super::STRING_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_STRING
        );

        assert_eq!(
            super::from_type_symbol(super::NUMBER_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_NUMBER
        );

        assert_eq!(
            super::from_type_symbol(super::OBJECT_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_OBJECT
        );

        assert_eq!(
            super::from_type_symbol(super::BOOLEAN_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_BOOLEAN
        );

        assert_eq!(
            super::from_type_symbol(super::BIGINT_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_BIGINT
        );

        assert_eq!(
            super::from_type_symbol(super::NULL_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_NULL
        );

        assert_eq!(
            super::from_type_symbol(super::UNDEFINED_SYMBOL.to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_NULL
        );

        assert_eq!(
            super::from_type_symbol("sss".to_string()),
            DataTypeEnum::DATA_TYPE_ENUM_UNSPECIFIED
        );
    }

    #[test]
    fn test_to_dataflow() {
        use proto::common::common::ResourceId;
        use proto::common::stream::DataflowMeta;
        use proto::common::stream::OperatorInfo;
        use std::collections::HashMap;

        let job_id = ResourceId::default();
        let mut op1 = OperatorInfo::default();
        op1.set_operator_id(0);
        let mut op2 = OperatorInfo::default();
        op2.set_operator_id(1);
        let operators = vec![op1.clone(), op2.clone()];

        let mut meta1 = DataflowMeta::default();
        meta1.set_center(0);
        meta1.set_neighbors(vec![1]);

        let dataflow = super::to_dataflow(&job_id, &operators, &[meta1.clone()]);

        let mut nodes = HashMap::default();
        nodes.insert(0, op1);
        nodes.insert(1, op2);

        assert_eq!(dataflow.get_job_id(), &job_id);
        assert_eq!(dataflow.get_meta(), &[meta1]);
        assert_eq!(dataflow.get_nodes(), &nodes);
    }

    #[test]
    fn test_validate_dataflow_success() {
        use crate::utils::validate_dataflow;
        use proto::common::stream::Dataflow;
        use proto::common::stream::DataflowMeta;
        use proto::common::stream::OperatorInfo;
        use protobuf::RepeatedField;
        use std::collections::HashMap;

        let mut dataflow = Dataflow::default();
        let mut meta = DataflowMeta::default();
        meta.set_center(0);
        meta.set_neighbors(vec![1, 2, 3]);

        dataflow.set_meta(RepeatedField::from_slice(&[meta]));

        let mut nodes = HashMap::default();
        let mut info_1 = OperatorInfo::default();
        info_1.set_operator_id(0);
        nodes.insert(0, info_1);
        dataflow.set_nodes(nodes.clone());

        let result = validate_dataflow(&dataflow);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            crate::err::DataflowValidateError::OperatorInfoMissing(_) => {}
            _ => panic!("unexpected error"),
        };

        (1..4).for_each(|index| {
            let mut info = OperatorInfo::default();
            info.set_operator_id(index);
            nodes.insert(index, info);
        });

        dataflow.set_nodes(nodes);
        let result = validate_dataflow(&dataflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataflow_is_cyclic() {
        use crate::utils::validate_dataflow;
        use proto::common::stream::Dataflow;
        use proto::common::stream::DataflowMeta;
        use proto::common::stream::OperatorInfo;
        use protobuf::RepeatedField;
        use std::collections::HashMap;

        let mut dataflow = Dataflow::default();
        let mut meta = DataflowMeta::default();
        meta.set_center(0);
        meta.set_neighbors(vec![1, 2, 3]);
        let mut meta_1 = DataflowMeta::default();
        meta_1.set_center(1);
        meta_1.set_neighbors(vec![2, 3, 4]);
        let mut meta_2 = DataflowMeta::default();
        meta_2.set_center(4);
        meta_2.set_neighbors(vec![0]);

        dataflow.set_meta(RepeatedField::from_slice(&[meta, meta_1, meta_2]));
        let mut nodes = HashMap::default();

        (0..5).for_each(|index| {
            let mut info = OperatorInfo::default();
            info.set_operator_id(index);
            nodes.insert(index, info);
        });

        dataflow.set_nodes(nodes);

        let result = validate_dataflow(&dataflow);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            crate::err::DataflowValidateError::CyclicDataflow => {}
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

use crate::net::hostname;
use proto::common::{
    common::JobId,
    stream::{Dataflow, DataflowMeta, OperatorInfo},
};
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

        env::args().for_each(|arg| {
            let is_key = arg.starts_with("-");
            if is_key {
                if !current_arg.is_empty() {
                    current_arg.clear();
                }
                current_arg.key = arg[1..arg.len()].to_string();
            } else {
                current_arg.value = arg.clone();
                let _ = map.insert(current_arg.key.clone(), current_arg.clone());
            }
        });

        Self { args: map.clone() }
    }
}

impl Args {
    pub fn arg(&self, flag: &str) -> Option<Arg> {
        let key = flag.to_string();
        self.args.get(&key).map(|val| val.clone())
    }
}

#[derive(Clone)]
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
        Err(_) => None,
    }
}

pub fn from_reader<R: std::io::Read>(reader: R) -> serde_json::Result<String> {
    let ref mut buf = Default::default();
    let mut buf_reader = std::io::BufReader::new(reader);
    buf_reader
        .read_to_string(buf)
        .map_err(|err| serde_json::Error::custom("fail to read from reader"))
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
    if operator.get_host_addr().get_host() == "localhost" {
        return false;
    }

    hostname()
        .map(|host| operator.get_host_addr().host != host)
        .unwrap_or(false)
}

pub fn to_dataflow(
    job_id: &JobId,
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

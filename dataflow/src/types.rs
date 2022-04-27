use std::{collections, string};

use crate::{err, types};
use crate::event;

pub type AdjacentList = Vec<AdjacentVec>;

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobID {
    pub table_id: String,
    pub header_id: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub row_idx: u64,
    pub value: Vec<u8>,
}

impl From<&data_client::tableflow::Entry> for Entry {
    fn from(entry: &data_client::tableflow::Entry) -> Self {
        Self {
            row_idx: entry.get_rowIdx(),
            value: entry.get_value().to_vec(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct AdjacentVec {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub neighbors: Vec<u64>,
    pub center: u64,
}

pub mod formula {
    use std::collections;
    use crate::types::ValueType;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct InitState {
        pub(crate) page: u64,
        pub(crate) limit: u64,
        pub(crate) done: bool,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
    pub struct FormulaGraph {
        #[serde(default)]
        pub meta: Vec<super::AdjacentVec>,
        #[serde(default)]
        pub data: collections::BTreeMap<String, FormulaOp>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    #[serde(tag = "type")]
    pub enum FormulaOp {
        #[serde(rename_all = "camelCase")]
        Reference {
            table_id: String,
            header_id: String,
            value_type: ValueType,
        },
        Add,
        Sum,
        Sumif
    }

    const REFERENCE_OP: &'static str = "Reference";

    impl common::KeyedValue<String, FormulaOp> for FormulaOp {
        fn key(&self) -> String {
            match self {
                FormulaOp::Reference { .. } => REFERENCE_OP.to_string(),
                _ => "".to_string()
            }
        }

        fn value(&self) -> FormulaOp {
            self.clone()
        }
    }

    impl FormulaOp {
        pub fn is_reference(&self) -> bool {
            match &self {
                FormulaOp::Reference { .. } => true,
                _ => false
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SourceDesc {
    Redis {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        db: usize,
    },
    Tableflow {
        host: String,
        port: usize,
        limit: u32,
        event_time: Option<u64>,
    },
}

pub fn job_id(table_id: &str, header_id: &str) -> JobID {
    JobID {
        table_id: table_id.to_string(),
        header_id: header_id.to_string(),
    }
}

pub fn traverse_from_bottom(meta: &Vec<AdjacentVec>) -> Vec<AdjacentVec> {
    let mut results = vec![];

    let mut grouped = common::lists::map_self(meta, |adj| adj.center.clone());

    common::lists::for_each(meta, |adj| {
        let mut flag = false;
        for id in &adj.neighbors {
            if grouped.contains_key(id) {
                flag = true;
                break;
            }
        }

        if !flag {
            grouped.remove(&adj.center);
            results.push(adj.clone());
        }
    });

    for (_, value) in grouped {
        results.push(value.clone())
    }

    results
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
pub struct Operator {
    pub addr: String,
    pub value: formula::FormulaOp,
    pub id: u64,
}

pub type NodeSet = collections::BTreeMap<String, Operator>;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GraphModel {
    pub job_id: JobID,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub meta: AdjacentList,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    #[serde(default)]
    pub nodes: NodeSet,
}

impl GraphModel {
    pub fn dispatch(&self) -> Result<(), err::CommonException> {
        let mut group = collections::HashMap::<String, Vec<&Operator>>::new();

        for (_, operator) in &self.nodes {
            match group.get(&operator.addr) {
                Some(ops) => {
                    let mut new_operators = vec![operator];
                    new_operators.extend(ops);
                    group.insert(operator.addr.clone(), new_operators);
                }
                None => {
                    group.insert(operator.addr.clone(), vec![operator]);
                }
            }
        }

        for (addr, ops) in group {
            let client = dataflow_api::worker::new_dataflow_worker_client(
                dataflow_api::worker::DataflowWorkerConfig {
                    host: None,
                    port: None,
                    uri: Some(addr),
                }
            );

            let ref graph_event = event::GraphEvent::ExecutionGraphSubmit {
                ops: types::GraphModel::from((&self.job_id, ops, &self.meta)),
                job_id: self.job_id.clone(),
            };

            let ref mut request = dataflow_api::dataflow_worker::ActionSubmitRequest::default();
            let result = serde_json::to_vec(graph_event)
                .map_err(|err| err::CommonException::from(err))
                .and_then(|value| {
                    request.set_value(value);
                    client.submit_action(request)
                        .map_err(|err| err::CommonException::from(err))
                })
                .map(|_| {
                    log::debug!("submit success")
                });

            if result.is_err() {
                return result;
            }
        }
        Ok(())
    }

    pub(crate) fn new(job_id: JobID,
                      meta: AdjacentList,
                      nodes: NodeSet) -> GraphModel {
        GraphModel {
            job_id,
            meta,
            nodes,
        }
    }
}

impl From<(&JobID, Vec<&Operator>, &AdjacentList)> for GraphModel {
    fn from(input: (&JobID, Vec<&Operator>, &AdjacentList)) -> Self {
        Self {
            job_id: input.0.clone(),
            meta: input.2.clone(),
            nodes: types::NodeSet::from_iter(
                input.1
                    .iter()
                    .map(
                        |op| (op.id.to_string(), (*op).clone())
                    )
            ),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum BinderType {
    Tableflow {
        page: u32
    },
    Redis,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Binder {
    pub job_id: JobID,
    pub binder_type: BinderType,
    pub table_id: String,
    pub header_id: String,
    pub id: u64,
    pub addr: String,
}

impl Binder {
    pub(crate) fn get_topic(&self) -> String {
        format!("{}/{}", &self.table_id, &self.header_id)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DataSourceEventType {
    TableflowTrigger {
        page: u32,
        limit: u32,
    },
    Delete,
    Update {
        old_value: Vec<u8>
    },
    Insert,
    Stop,
    Invalid,
}

impl From<&ConnectorEventType> for DataSourceEventType {
    fn from(t: &ConnectorEventType) -> Self {
        match t {
            ConnectorEventType::Action(action) => {
                match action {
                    ActionType::INSERT => Self::Insert,
                    ActionType::UPDATE {
                        old_value
                    } => Self::Update {
                        old_value: old_value.clone()
                    },
                    ActionType::DELETE => Self::Delete,
                    _ => Self::Invalid
                }
            }
            ConnectorEventType::Close => Self::Stop
        }
    }
}

#[derive(Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE {
        old_value: Vec<u8>
    },
    DELETE,
    INVALID,
}

impl From<DataSourceEventType> for ActionType {
    fn from(event_type: DataSourceEventType) -> Self {
        match event_type {
            DataSourceEventType::TableflowTrigger { .. } => Self::INSERT,
            DataSourceEventType::Delete => Self::DELETE,
            DataSourceEventType::Update {
                old_value
            } => Self::UPDATE { old_value },
            DataSourceEventType::Insert => Self::INSERT,
            DataSourceEventType::Stop => Self::INVALID,
            DataSourceEventType::Invalid => Self::INVALID
        }
    }
}

impl ActionType {
    pub fn is_value_update(&self) -> bool {
        match self {
            ActionType::INSERT => true,
            ActionType::UPDATE { .. } => true,
            _ => false,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
#[serde(tag = "type", content = "action")]
pub enum ConnectorEventType {
    Action(ActionType),
    Close,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
pub enum ValueType {
    String,
    UnsignedInt,
    Double,
    Float,
    UnsignedLong,
    UnsignedLongLong,
    Int,
    Long,
    LongLong,
}

impl From<TypedValue> for ValueType {
    fn from(typed: TypedValue) -> Self {
        match typed {
            TypedValue::String(_) => Self::String,
            TypedValue::UnsignedInt(_) => Self::UnsignedInt,
            TypedValue::Double(_) => Self::Double,
            TypedValue::Float(_) => Self::Float,
            TypedValue::UnsignedLong(_) => Self::UnsignedLong,
            TypedValue::UnsignedLongLong(_) => Self::UnsignedLongLong,
            TypedValue::Int(_) => Self::Int,
            TypedValue::Long(_) => Self::Long,
            TypedValue::LongLong(_) => Self::LongLong
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum TypedValue {
    String(String),
    UnsignedInt(u32),
    Double(f64),
    Float(f32),
    UnsignedLong(u64),
    UnsignedLongLong(u128),
    Int(i32),
    Long(i64),
    LongLong(i128),
}

impl TypedValue {
    pub fn get_data(&self) -> Vec<u8> {
        match self {
            TypedValue::String(value) => value.as_bytes().to_vec(),
            TypedValue::UnsignedInt(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::Double(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::Float(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::UnsignedLong(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::UnsignedLongLong(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::Int(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::Long(value) => value.clone().to_be_bytes().to_vec(),
            TypedValue::LongLong(value) => value.clone().to_be_bytes().to_vec()
        }
    }

    pub fn get_type(&self) -> ValueType {
        match self {
            TypedValue::String(_) => ValueType::String,
            TypedValue::UnsignedInt(_) => ValueType::UnsignedInt,
            TypedValue::Double(_) => ValueType::Double,
            TypedValue::Float(_) => ValueType::Float,
            TypedValue::UnsignedLong(_) => ValueType::UnsignedLong,
            TypedValue::UnsignedLongLong(_) => ValueType::UnsignedLongLong,
            TypedValue::Int(_) => ValueType::Int,
            TypedValue::Long(_) => ValueType::Long,
            TypedValue::LongLong(_) => ValueType::LongLong
        }
    }
}

#[derive(Clone)]
pub struct ActionValue {
    pub action: ActionType,
    pub value: TypedValue,
    pub from: u64,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct ValueState {
    values: collections::BTreeMap<u64, TypedValue>,
}

impl ValueState {
    pub fn new(values: collections::BTreeMap<u64, TypedValue>) -> ValueState {
        ValueState {
            values
        }
    }
}

impl FromBytes for ValueState {
    fn from_bytes(data: Vec<u8>) -> Option<Self> {
        match serde_json::from_slice::<ValueState>(data.as_slice()) {
            Ok(value) => Some(value),
            Err(err) => {
                log::error!("deserialize failed {}", err);
                None
            }
        }
    }

    fn to_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(result) => result,
            Err(_) => Default::default()
        }
    }
}

impl ValueState {
    pub fn remove(&mut self, from: &u64) -> Option<TypedValue> {
        self.values.remove(from)
    }

    pub fn update(&mut self, from: &u64, value: &TypedValue) -> Option<TypedValue> {
        self.values.insert(from.clone(), value.clone())
    }
}

pub type RowIdx = u64;

pub trait FromBytes: Sized {
    fn from_bytes(data: Vec<u8>) -> Option<Self>;
    fn to_string(&self) -> String;
}

use std::{collections, ops, string};
use std::cmp::Ordering;
use crate::{err, event, lists};

use bytes::Buf;

pub type AdjacentList = Vec<AdjacentVec>;
pub type DataTypeSymbol = u8;

pub(crate) const STRING: DataTypeSymbol = 1;
pub(crate) const INT: DataTypeSymbol = 2;
pub(crate) const LONG: DataTypeSymbol = 3;
pub(crate) const FLOAT: DataTypeSymbol = 4;
pub(crate) const DOUBLE: DataTypeSymbol = 5;
pub(crate) const BOOLEAN: DataTypeSymbol = 6;

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

impl Into<data_client::tableflow::Entry> for Entry {
    fn into(self) -> data_client::tableflow::Entry {
        let mut entry = data_client::tableflow::Entry::new();
        entry.set_value(self.value.clone());
        entry.set_rowIdx(self.row_idx);
        entry
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Eq, PartialEq)]
pub struct AdjacentVec {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub neighbors: Vec<u64>,
    pub center: u64,
}

pub mod formula {
    use std::collections;

    use super::ValueType;

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

    impl FormulaGraph {
        pub fn find_upstreams(&self, node_id: super::NodeIdx) -> Vec<super::NodeIdx> {
            let mut results = vec![];
            self.meta.iter().for_each(|adj| {
                if adj.neighbors.contains(&node_id) {
                    results.push(adj.center);
                }
            });
            results
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    pub struct ValueOp {
        pub value: Vec<u8>,
        pub node_id: super::NodeIdx,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    #[serde(tag = "type")]
    pub enum FormulaOp {
        #[serde(rename_all = "camelCase")]
        Reference {
            table_id: String,
            header_id: String,
        },
        Add {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Sum,
        Sumif,
        Countif,
        Count,
        Avg,
        Group,
        Groupif,
        Max,
        Maxif,
        Min,
        Minif,
        Xlookup,
        Sub {
            values: Vec<ValueOp>
        },
        Mul {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Div {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Eq {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Neq {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Lt {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Gt {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Lte {
            values: Vec<ValueOp>
        },
        Gte {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        And {
            #[serde(default)]
            values: Vec<ValueOp>
        },
        Or {
            #[serde(default)]
            values: Vec<ValueOp>
        },
    }

    const REFERENCE_OP: &'static str = "Reference";

    impl super::KeyedValue<String, FormulaOp> for FormulaOp {
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
    Tableflow {
        host: String,
        port: usize,
        event_time: Option<u64>,
    },
    Kafka {
        brokers: Vec<String>,
        topic: String,
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

    let mut grouped = lists::map_self(meta, |adj| adj.center.clone());

    lists::for_each(meta, |adj| {
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
    #[serde(default)]
    pub upstream: Vec<NodeIdx>,
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
                ops: GraphModel::from((&self.job_id, ops, &self.meta)),
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

    pub fn new(job_id: JobID,
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
            nodes: NodeSet::from_iter(
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
    Update,
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
                    ActionType::UPDATE => Self::Update,
                    ActionType::DELETE => Self::Delete,
                    _ => Self::Invalid
                }
            }
            ConnectorEventType::Close => Self::Stop
        }
    }
}

#[derive(Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize, Debug, Hash)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
    INVALID,
}

impl From<DataSourceEventType> for ActionType {
    fn from(event_type: DataSourceEventType) -> Self {
        match event_type {
            DataSourceEventType::TableflowTrigger { .. } => Self::INSERT,
            DataSourceEventType::Delete => Self::DELETE,
            DataSourceEventType::Update => Self::UPDATE,
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
    Double,
    Float,
    Int,
    Long,
    Boolean,
    Invalid,
}

impl From<TypedValue> for ValueType {
    fn from(typed: TypedValue) -> Self {
        match typed {
            TypedValue::String(_) => Self::String,
            TypedValue::Double(_) => Self::Double,
            TypedValue::Float(_) => Self::Float,
            TypedValue::Int(_) => Self::Int,
            TypedValue::Long(_) => Self::Long,
            TypedValue::Boolean(_) => Self::Boolean,
            _ => Self::Invalid
        }
    }
}

impl From<ValueType> for DataTypeSymbol {
    fn from(vt: ValueType) -> Self {
        match vt {
            ValueType::String => STRING,
            ValueType::Double => DOUBLE,
            ValueType::Float => FLOAT,
            ValueType::Int => INT,
            ValueType::Long => LONG,
            ValueType::Boolean => BOOLEAN,
            ValueType::Invalid => 7
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum TypedValue {
    String(String),
    Double(f64),
    Float(f32),
    Int(i32),
    Long(i64),
    Boolean(bool),
    Invalid,
}

impl Eq for TypedValue {}

// TODO refactor by macro in future
impl PartialEq for TypedValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TypedValue::String(value) => match other {
                TypedValue::String(other) => value == other,
                _ => false
            }
            TypedValue::Double(value) => match other {
                TypedValue::Double(other) => value.eq(other),
                TypedValue::Float(other) => value.eq(&(*other as f64)),
                TypedValue::Int(other) => value.eq(&(*other as f64)),
                TypedValue::Long(other) => value.eq(&(*other as f64)),
                _ => false
            }
            TypedValue::Float(value) => match other {
                TypedValue::Double(other) => (*value as f64).eq(other),
                TypedValue::Float(other) => value.eq(other),
                TypedValue::Int(other) => value.eq(&(*other as f32)),
                TypedValue::Long(other) => (*value as f64).eq(&(*other as f64)),
                _ => false
            }
            TypedValue::Int(value) => match other {
                TypedValue::Double(other) => (*value as f64).eq(other),
                TypedValue::Float(other) => other.eq(&(*value as f32)),
                TypedValue::Int(other) => value.eq(other),
                TypedValue::Long(other) => other.eq(&(*value as i64)),
                _ => false
            }
            TypedValue::Long(value) => match other {
                TypedValue::Double(other) => (*value as f64).eq(other),
                TypedValue::Float(other) => (*value as f64).eq(&(*other as f64)),
                TypedValue::Int(other) => value.eq(&(*other as i64)),
                TypedValue::Long(other) => value == other,
                _ => false
            }
            TypedValue::Boolean(value) => match other {
                TypedValue::Boolean(other) => value == other,
                _ => false
            }
            TypedValue::Invalid => match other {
                TypedValue::Invalid => true,
                _ => false
            }
        }
    }
}

// TODO refactor by macro in future
impl PartialOrd for TypedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            TypedValue::String(value) => match other {
                TypedValue::String(other) => value.partial_cmp(other),
                _ => None
            }
            TypedValue::Double(value) => match other {
                TypedValue::Double(other) => value.partial_cmp(other),
                TypedValue::Float(other) => value.partial_cmp(&(*other as f64)),
                TypedValue::Int(other) => value.partial_cmp(&(*other as f64)),
                TypedValue::Long(other) => value.partial_cmp(&(*other as f64)),
                _ => None
            }
            TypedValue::Float(value) => match other {
                TypedValue::Double(other) => (*value as f64).partial_cmp(other),
                TypedValue::Float(other) => value.partial_cmp(other),
                TypedValue::Int(other) => value.partial_cmp(&(*other as f32)),
                TypedValue::Long(other) => (*value as f64).partial_cmp(&(*other as f64)),
                _ => None
            }
            TypedValue::Int(value) => match other {
                TypedValue::Double(other) => (*value as f64).partial_cmp(other),
                TypedValue::Float(other) => (*value as f32).partial_cmp(other),
                TypedValue::Int(other) => value.partial_cmp(other),
                TypedValue::Long(other) => (*value as i64).partial_cmp(other),
                _ => None
            }
            TypedValue::Long(value) => match other {
                TypedValue::Double(other) => (*value as f64).partial_cmp(other),
                TypedValue::Float(other) => (*value as f64).partial_cmp(&(*other as f64)),
                TypedValue::Int(other) => value.partial_cmp(&(*other as i64)),
                TypedValue::Long(other) => value.partial_cmp(other),
                _ => None
            }
            _ => None
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
            TypedValue::Double(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double(value - other),
                TypedValue::Float(other) => TypedValue::Double(value - (other as f64)),
                TypedValue::Int(other) => TypedValue::Double(value - (other as f64)),
                TypedValue::Long(other) => TypedValue::Double(value - (other as f64)),
                _ => TypedValue::Invalid
            }
            TypedValue::Float(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64) - other),
                TypedValue::Float(other) => TypedValue::Float(value - other),
                TypedValue::Int(other) => TypedValue::Float(value - (other as f32)),
                TypedValue::Long(other) => TypedValue::Double((value as f64) - (other as f64)),
                _ => TypedValue::Invalid
            }
            TypedValue::Int(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64) - other),
                TypedValue::Float(other) => TypedValue::Float((value as f32) - other),
                TypedValue::Int(other) => TypedValue::Int(value - other),
                TypedValue::Long(other) => TypedValue::Long((value as i64) - other),
                _ => TypedValue::Invalid
            }
            TypedValue::Long(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64) - other),
                TypedValue::Float(other) => TypedValue::Double((value as f64) - (other as f64)),
                TypedValue::Int(other) => TypedValue::Long(value - (other as i64)),
                TypedValue::Long(other) => TypedValue::Long(value - other),
                _ => TypedValue::Invalid
            }
            _ => TypedValue::Invalid
        }
    }
}

// TODO refactor by macro in future
impl ops::Div for TypedValue {
    type Output = TypedValue;

    fn div(self, rhs: Self) -> Self::Output {
        todo!()
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
            TypedValue::Double(value) => {
                match rhs {
                    TypedValue::Double(other) => TypedValue::Double(other * value),
                    TypedValue::Float(other) => TypedValue::Double((other as f64 * value) as f64),
                    TypedValue::Int(other) => TypedValue::Double(other as f64 * value),
                    TypedValue::Long(other) => TypedValue::Double(other as f64 * value),
                    _ => self.clone(),
                }
            }
            TypedValue::Float(value) => {
                match rhs {
                    TypedValue::Double(other) => TypedValue::Double(other * value as f64),
                    TypedValue::Float(other) => TypedValue::Float(other * value),
                    TypedValue::Int(other) => TypedValue::Float(other as f32 * value),
                    TypedValue::Long(other) => TypedValue::Double(other as f64 * value as f64),
                    _ => self.clone(),
                }
            }
            TypedValue::Int(value) => {
                match rhs {
                    TypedValue::Double(other) => TypedValue::Double(other * value as f64),
                    TypedValue::Float(other) => TypedValue::Float(other * value as f32),
                    TypedValue::Int(other) => TypedValue::Int(other * value),
                    TypedValue::Long(other) => TypedValue::Long(other * value as i64),
                    _ => self.clone(),
                }
            }
            TypedValue::Long(value) => {
                match rhs {
                    TypedValue::Double(other) => TypedValue::Double(other * value as f64),
                    TypedValue::Float(other) => TypedValue::Double(other as f64 * value as f64),
                    TypedValue::Int(other) => TypedValue::Long(other as i64 * value),
                    TypedValue::Long(other) => TypedValue::Long(other * value),
                    _ => self.clone(),
                }
            }
            _ => TypedValue::Invalid
        }
    }
}

// TODO refactor by macro in future
impl ops::Add for TypedValue {
    type Output = TypedValue;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            TypedValue::String(value) => {
                match rhs {
                    TypedValue::String(other) =>
                        TypedValue::String(value.add(other.as_str())),
                    _ => TypedValue::Invalid
                }
            }
            TypedValue::Double(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double(value.add(other)),
                TypedValue::Float(other) => TypedValue::Double(value.add(other as f64)),
                TypedValue::Int(other) => TypedValue::Double(value.add(other as f64)),
                TypedValue::Long(other) => TypedValue::Double(value.add(other as f64)),
                _ => TypedValue::Invalid
            },
            TypedValue::Float(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64).add(other)),
                TypedValue::Float(other) => TypedValue::Float(value.add(other)),
                TypedValue::Int(other) => TypedValue::Float(value.add(other as f32)),
                TypedValue::Long(other) => TypedValue::Double((value as f64).add(other as f64)),
                _ => TypedValue::Invalid
            },
            TypedValue::Int(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64).add(other)),
                TypedValue::Float(other) => TypedValue::Float((value as f32).add(other)),
                TypedValue::Int(other) => TypedValue::Int(value.add(other)),
                TypedValue::Long(other) => TypedValue::Long((value as i64).add(other)),
                _ => TypedValue::Invalid
            }
            TypedValue::Long(value) => match rhs {
                TypedValue::Double(other) => TypedValue::Double((value as f64).add(other)),
                TypedValue::Float(other) => TypedValue::Double((value as f64).add(other as f64)),
                TypedValue::Int(other) => TypedValue::Long((value).add(other as i64)),
                TypedValue::Long(other) => TypedValue::Long(value.add(other)),
                _ => TypedValue::Invalid
            }
            _ => TypedValue::Invalid
        }
    }
}

// TODO refactor by macro in future
impl ops::AddAssign for TypedValue {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            TypedValue::String(value) => {
                match rhs {
                    TypedValue::String(other) =>
                        value.push_str(other.as_str()),
                    TypedValue::Double(other) =>
                        value.push_str(other.to_string().as_str()),
                    TypedValue::Float(other) =>
                        value.push_str(other.to_string().as_str()),
                    TypedValue::Int(other) =>
                        value.push_str(other.to_string().as_str()),
                    TypedValue::Long(other) =>
                        value.push_str(other.to_string().as_str()),
                    _ => {}
                }
            }
            TypedValue::Double(value) => match rhs {
                TypedValue::Double(other) => value.add_assign(other),
                _ => {}
            },
            TypedValue::Float(value) => match rhs {
                TypedValue::Float(other) => value.add_assign(other),
                _ => {}
            },
            TypedValue::Int(value) => match rhs {
                TypedValue::Int(other) => value.add_assign(other),
                _ => {}
            }
            TypedValue::Long(value) => match rhs {
                TypedValue::Long(other) => value.add_assign(other),
                _ => {}
            }
            _ => {}
        }
    }
}

impl TypedValue {
    pub fn get_data(&self) -> Vec<u8> {
        let symbol: DataTypeSymbol = self.get_type().into();

        match self {
            TypedValue::String(value) => {
                let mut result = value.as_bytes().to_vec();
                result.insert(0, symbol);
                result
            }
            TypedValue::Double(value) => {
                let mut result = value.to_be_bytes().to_vec();
                result.insert(0, symbol);
                result
            }
            TypedValue::Float(value) => {
                let mut result = value.to_be_bytes().to_vec();
                result.insert(0, symbol);
                result
            }
            TypedValue::Int(value) => {
                let mut result = value.to_be_bytes().to_vec();
                result.insert(0, symbol);
                result
            }
            TypedValue::Long(value) => {
                let mut result = value.to_be_bytes().to_vec();
                result.insert(0, symbol);
                result
            }
            TypedValue::Boolean(value) => {
                let data = if *value { 1 as u8 } else { 0 as u8 };

                vec![symbol, data]
            }
            _ => vec![]
        }
    }

    pub fn from(data: &Vec<u8>) -> TypedValue {
        let data_type = data[0];

        match data_type {
            STRING => match String::from_utf8(data[1..data.len()].to_vec()) {
                Ok(val) => TypedValue::String(val),
                Err(_) => TypedValue::Invalid
            },
            INT => TypedValue::Int(data[1..data.len()].to_vec().as_slice().get_i32()),
            FLOAT => TypedValue::Float(data[1..data.len()].to_vec().as_slice().get_f32()),
            LONG => TypedValue::Long(data[1..data.len()].to_vec().as_slice().get_i64()),
            DOUBLE => TypedValue::Double(data[1..data.len()].to_vec().as_slice().get_f64()),
            BOOLEAN => TypedValue::Boolean(data[1] == 1),
            _ => TypedValue::Invalid
        }
    }

    pub fn get_type(&self) -> ValueType {
        match self {
            TypedValue::String(_) => ValueType::String,
            TypedValue::Double(_) => ValueType::Double,
            TypedValue::Float(_) => ValueType::Float,
            TypedValue::Int(_) => ValueType::Int,
            TypedValue::Long(_) => ValueType::Long,
            _ => ValueType::Invalid
        }
    }
}

#[derive(Clone)]
pub struct ActionValue {
    pub action: ActionType,
    pub value: TypedValue,
    pub from: NodeIdx,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct ValueState {
    pub value: TypedValue,
    pub node_idx: NodeIdx,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct FormulaState {
    pub value: Vec<u8>,
    pub node_states: Vec<ValueState>,
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
        todo!()
    }

    pub fn update(&self, from: &u64, value: &TypedValue) {
        todo!()
    }
}

pub type RowIdx = u64;
pub type NodeIdx = u64;

pub trait FromBytes: Sized {
    fn from_bytes(data: Vec<u8>) -> Option<Self>;
    fn to_string(&self) -> String;
}

pub trait KeyedValue<K, V> {
    fn key(&self) -> K;
    fn value(&self) -> V;
}

use std::collections;

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct JobID {
    pub table_id: String,
    pub header_id: String,
}

pub struct NodeException {}

pub type RowData = collections::BTreeMap<u64, String>;
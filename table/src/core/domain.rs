use std::collections::BTreeMap;
use std::time;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;

use dataflow::types::DataflowDesc;

use crate::core::err;
use core::view;

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    #[serde(rename(serialize = "headerId"))]
    header_id: u32,
    #[serde(rename(serialize = "type"))]
    _type: HeaderType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "format", content = "data")]
pub enum HeaderType {
    String(String),
    Formula(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TableType {
    View {
        dataflow: DataflowDesc<view::table::TableGraph>
    },
    Base,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableSchema {
    name: String,
    id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    header: Vec<Header>,
    code: String,
    #[serde(rename(serialize = "creationTime"))]
    creation_time: time::SystemTime,
    #[serde(rename(serialize = "type"))]
    _type: TableType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableRowRecord {
    data: BTreeMap<u32, String>,
    #[serde(rename(serialize = "tableId"))]
    table_id: String,
    #[serde(rename(serialize = "creationTime"))]
    creation_time: time::SystemTime,
}

pub fn new_row_record(data: &Vec<u8>, id: &String) -> Result<TableRowRecord, err::TableError> {
    match serde_json::from_slice::<BTreeMap<u32, String>>(data.as_slice()) {
        Ok(map) => Ok(TableRowRecord {
            data: map,
            table_id: id.clone(),
            creation_time: time::SystemTime::now(),
        }),
        Err(err) => Err(err::TableError::InvalidJson(err))
    }
}

impl TableRowRecord {
    pub fn data(&self) -> &BTreeMap<u32, String> {
        &self.data
    }

    pub fn id(self) -> String {
        self.table_id.clone()
    }

    pub fn to_column(&self) -> Vec<TableColumnRecord> {
        let table_id = &self.table_id;

        self.data.iter()
            .map(|(id, val)| TableColumnRecord::new(id, val, table_id))
            .collect()
    }
}


#[derive(Debug, Clone)]
pub struct TableColumnRecord {
    data: Vec<String>,
    col_id: u32,
    table_id: String,
    creation_time: time::SystemTime,
}

impl Serialize for TableColumnRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = serializer.serialize_struct("TableColumnRecord", 4)?;
        todo!()
    }
}

impl<'de> Deserialize<'de> for TableColumnRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }
}

impl TableColumnRecord {
    pub fn new(header_id: &u32, value: &String, table_id: &String) -> TableColumnRecord {
        TableColumnRecord {
            data: vec![value.to_string()],
            col_id: header_id.clone(),
            table_id: table_id.clone(),
            creation_time: time::SystemTime::now(),
        }
    }

    pub fn col_id(&self) -> &u32 {
        &self.col_id
    }

    pub fn table_id(&self) -> String {
        self.table_id.clone()
    }

    pub fn data(&self) -> &Vec<String> {
        &self.data
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableColumnUpdateLog {
    records: Vec<TableColumnRecord>,
    #[serde(rename(serialize = "creationTime"))]
    creation_time: time::SystemTime,
    archived: bool,
}

impl TableColumnUpdateLog {
    pub fn new(records: Vec<TableColumnRecord>) -> TableColumnUpdateLog {
        TableColumnUpdateLog {
            records,
            creation_time: time::SystemTime::now(),
            archived: false,
        }
    }
}
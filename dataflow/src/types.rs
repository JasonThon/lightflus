use std::collections;

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct JobID {
    pub table_id: String,
    pub header_id: String,
}

pub struct NodeException {}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub row_idx: u64,
    pub value: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct AdjacentVec {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub neighbors: Vec<u64>,
    pub center: u64,
}

pub mod formula {
    use std::collections;

    use crate::{graph, types};
    use crate::source;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct InitState {
        pub(crate) page: u64,
        pub(crate) limit: u64,
        pub(crate) done: bool,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
    pub struct FormulaGraph {
        pub meta: Vec<super::AdjacentVec>,
        pub data: collections::BTreeMap<u64, FormulaOp>,
    }

    #[serde(tag = "type")]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum FormulaOp {
        Reference {
            #[serde(rename(serialize = "tableId"))]
            table_id: String,
            #[serde(rename(serialize = "headerId"))]
            header_id: String,
            sources: Vec<types::SourceDesc>,
        },
        Add,
    }

    const REFERENCE_OP: &'static str = "Reference";

    impl core::KeyedValue<String, FormulaOp> for FormulaOp {
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
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum SourceDesc {
    Redis {
        host: String,
        port: u16,
        username: String,
        password: String,
        db: usize,
    },
    Tableflow {
        host: String,
        port: usize,
        page: u32,
        limit: u32,
    },
}

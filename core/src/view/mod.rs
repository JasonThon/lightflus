pub mod table {
    use std::collections;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct TableGraph {
        pub meta: Vec<AdjacentVec>,
        pub data: collections::BTreeMap<u32, FormulaOp>
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct AdjacentVec {
        #[serde(rename(serialize = "nodeId"))]
        pub node_id: u32,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub neighbors: Vec<u32>
    }

    #[serde(tag = "type")]
    pub enum FormulaOp {
        Reference {
            #[serde(rename(serialize = "tableId"))]
            table_id: String,
            #[serde(rename(serialize = "headerId"))]
            header_id: String
        }
    }

}
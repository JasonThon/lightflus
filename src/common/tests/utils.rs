use std::collections;
use common::types;

pub fn default_adj_vec() -> Vec<types::AdjacentVec> {
    vec![
        types::AdjacentVec {
            neighbors: vec![1, 2, 3],
            center: 0,
        },
        types::AdjacentVec {
            neighbors: vec![4, 5],
            center: 1,
        },
        types::AdjacentVec {
            neighbors: vec![5, 6],
            center: 2,
        },
        types::AdjacentVec {
            neighbors: vec![7],
            center: 3,
        },
        types::AdjacentVec {
            neighbors: vec![8],
            center: 6,
        },
    ]
}

pub fn default_formula_graph() -> types::stream::StreamGraph {
    let mut values = vec![];

    for (id, operator) in default_node_set().iter() {
        values.push((id.clone(), operator.value.clone()))
    }

    types::stream::StreamGraph {
        meta: default_adj_vec(),
        data: collections::BTreeMap::from_iter(values),
    }
}

pub fn default_node_set() -> types::NodeSet {
    types::NodeSet::from(
        [
            ("0".to_string(), types::OperatorInfo {
                addr: "localhost".to_string(),
                value: types::stream::OperatorType::Reference {
                    table_id: "tableId_1".to_string(),
                    header_id: "headerId_1".to_string(),
                },
                id: 0,
                upstream: vec![]
            }),
            ("1".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 1,
                upstream: vec![]
            }),
            ("2".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 2,
                upstream: vec![]
            }),
            ("3".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 3,
                upstream: vec![]
            }),
            ("4".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 4,
                upstream: vec![]
            }),
            ("5".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 5,
                upstream: vec![]
            }),
            ("6".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 6,
                upstream: vec![]
            }),
            ("7".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 6,
                upstream: vec![]
            }),
            ("8".to_string(), types::OperatorInfo {
                addr: "".to_string(),
                value: types::stream::OperatorType::Sum,
                id: 8,
                upstream: vec![]
            }),
        ]
    )
}


pub fn default_empty_graph() -> types::DataflowContext {
    types::DataflowContext {
        job_id: types::job_id("tableId", "headerId"),
        meta: vec![],
        nodes: Default::default(),
    }
}
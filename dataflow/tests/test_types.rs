use dataflow::{event, types};
use std::collections;

mod utils;

use utils::*;

#[test]
fn test_serialize_empty_graph() {
    let ref model = default_empty_graph();
    let result = serde_json::to_string(model);
    assert!(result.is_ok());

    let value = result.unwrap();
    let deserialize_model = serde_json::from_str::<types::GraphModel>(value.as_str());
    assert!(deserialize_model.is_ok());

    let ref deser_model = deserialize_model.unwrap();
    assert_eq!(&deser_model.job_id, &model.job_id);
    assert!((&deser_model.nodes).is_empty());
    assert!((&deser_model.meta).is_empty());
}

#[test]
fn test_traverse_from_bottom() {
    let ref meta = vec![
        types::AdjacentVec {
            center: 4,
            neighbors: vec![8, 9],
        },
        types::AdjacentVec {
            center: 0,
            neighbors: vec![1, 2, 3],
        },
        types::AdjacentVec {
            center: 1,
            neighbors: vec![4, 5],
        },
        types::AdjacentVec {
            center: 3,
            neighbors: vec![5, 7],
        },
    ];

    let traversed = types::traverse_from_bottom(meta);

    assert_eq!(traversed, vec![
        types::AdjacentVec {
            center: 4,
            neighbors: vec![8, 9],
        },
        types::AdjacentVec {
            center: 1,
            neighbors: vec![4, 5],
        },
        types::AdjacentVec {
            center: 3,
            neighbors: vec![5, 7],
        },
        types::AdjacentVec {
            center: 0,
            neighbors: vec![1, 2, 3],
        },
    ])
}

#[test]
fn test_job_id_eq() {
    let id = types::job_id("tableId", "headerId");
    let id1 = types::job_id("tableId", "headerId");
    assert_eq!(id, id1);
}

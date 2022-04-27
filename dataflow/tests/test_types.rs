extern crate core;

use dataflow::{event, types};
use std::collections;
use bytes::Buf;

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

#[test]
fn test_typed_value() {
    let long_long = types::TypedValue::LongLong(1 << 120);
    assert_eq!(long_long.get_data().as_slice().get_i128(), 1 << 120);
    assert_eq!(long_long.get_type(), types::ValueType::LongLong);

    let int = types::TypedValue::Int(1 << 30);
    assert_eq!(int.get_data().as_slice().get_i32(), 1 << 30);
    assert_eq!(int.get_type(), types::ValueType::Int);

    let double = types::TypedValue::Double(1.6546);
    assert_eq!(double.get_data().as_slice().get_f64(), 1.6546);
    assert_eq!(double.get_type(), types::ValueType::Double);

    let unsigned_long_long = types::TypedValue::UnsignedLongLong(1 << 120);
    assert_eq!(unsigned_long_long.get_type(), types::ValueType::UnsignedLongLong);
    assert_eq!(unsigned_long_long.get_data().as_slice().get_u128(), 1 << 120);

    let unsigned_long = types::TypedValue::UnsignedLong(1 << 60);
    assert_eq!(unsigned_long.get_type(), types::ValueType::UnsignedLong);
    assert_eq!(unsigned_long.get_data().as_slice().get_u64(), 1 << 60);

    let unsigned_int = types::TypedValue::UnsignedInt(1 << 30);
    assert_eq!(unsigned_int.get_type(), types::ValueType::UnsignedInt);
    assert_eq!(unsigned_int.get_data().as_slice().get_u32(), 1 << 30);

    let long = types::TypedValue::Long(1 << 60);
    assert_eq!(long.get_type(), types::ValueType::Long);
    assert_eq!(long.get_data().as_slice().get_u64(), 1 << 60);

    let float = types::TypedValue::Float(1.6546);
    assert_eq!(float.get_type(), types::ValueType::Float);
    assert_eq!(float.get_data().as_slice().get_f32(), 1.6546);

    let string = types::TypedValue::String("test".to_string());
    assert_eq!(string.get_type(), types::ValueType::String);
    assert_eq!(std::string::String::from_utf8(string.get_data()), Ok("test".to_string()))
}

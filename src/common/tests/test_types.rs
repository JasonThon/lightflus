extern crate core;

use common::{event, types};
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
fn test_typed_value_get_data() {
    let int = types::TypedValue::Int(1 << 30);
    assert_eq!(int.get_data().as_slice().get_i32(), 1 << 30);
    assert_eq!(int.get_type(), types::ValueType::Int);

    let double = types::TypedValue::Double(1.6546);
    assert_eq!(double.get_data().as_slice().get_f64(), 1.6546);
    assert_eq!(double.get_type(), types::ValueType::Double);

    let long = types::TypedValue::Long(1 << 60);
    assert_eq!(long.get_type(), types::ValueType::Long);
    assert_eq!(long.get_data().as_slice().get_u64(), 1 << 60);

    let float = types::TypedValue::Float(1.6546);
    assert_eq!(float.get_type(), types::ValueType::Float);
    assert_eq!(float.get_data().as_slice().get_f32(), 1.6546);

    let string = types::TypedValue::String("test".to_string());
    assert_eq!(string.get_type(), types::ValueType::String);
    assert_eq!(String::from_utf8(string.get_data()), Ok("test".to_string()))
}

#[test]
fn test_typed_value_left_int_dual_op() {
    std::env::set_var("double.accuracy", 8.to_string());
    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Int(200);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Int(300));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Int(-100));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Int(20000));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.5));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Long(3000);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Long(3100));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Long(-2900));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Long(300000));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.03333333));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Float(313.129);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Float(413.129));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Float(-213.129));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Double(31300.129));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.3193572));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Double(313.12998000);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Double(413.12998));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Double(-213.129));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Double(31312.998));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.3193572));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Invalid;
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);
}

#[test]
fn test_typed_value_left_long_dual_op() {
    std::env::set_var("double.accuracy", 8.to_string());
    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Long(2000);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Long(3000));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Long(-1000));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Long(2000000));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.5));

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Float(229.102);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Double(1129.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Double(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Double(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.04281188));

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Double(229.102);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Double(1129.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Double(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Double(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Double(0.04281188));

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Invalid;
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);
}

#[test]
fn test_typed_value_left_float_dual_op() {
    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::Float(899.9999);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Float(2899.11112233));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Float(1099.11132233));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);
}
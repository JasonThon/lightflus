use common::types;

use bytes::Buf;

#[test]
fn test_typed_value_get_data() {
    let int = types::TypedValue::Int(1 << 30);
    let mut data = int.get_data();
    data.remove(0);
    assert_eq!(data.as_slice().get_i32(), 1 << 30);
    assert_eq!(int.get_type(), types::ValueType::Int);

    let double = types::TypedValue::Double(1.6546);
    let mut data = double.get_data();
    data.remove(0);
    assert_eq!(data.as_slice().get_f64(), 1.6546);
    assert_eq!(double.get_type(), types::ValueType::Double);

    let long = types::TypedValue::Long(1 << 60);
    assert_eq!(long.get_type(), types::ValueType::Long);
    let mut data = long.get_data();
    data.remove(0);
    assert_eq!(data.as_slice().get_u64(), 1 << 60);

    let float = types::TypedValue::Float(1.6546);
    assert_eq!(float.get_type(), types::ValueType::Float);
    let mut data = float.get_data();
    data.remove(0);
    assert_eq!(data.as_slice().get_f32(), 1.6546);

    let string = types::TypedValue::String("test".to_string());
    assert_eq!(string.get_type(), types::ValueType::String);
    let mut data = string.get_data();
    data.remove(0);
    assert_eq!(String::from_utf8(data), Ok("test".to_string()))
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
    assert_eq!(a1 / a2, types::TypedValue::Double(0.03333333333333333));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Float(313.129);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Float(413.129));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Float(-213.129));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Float(31312.9));
    assert_eq!(a1 / a2, types::TypedValue::Float(0.3193572));

    let a1 = types::TypedValue::Int(100);
    let a2 = types::TypedValue::Double(313.12998000);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Double(413.12998)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Double(-213.12998)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Double(31312.998)
    );
    assert_eq!(a1 / a2, types::TypedValue::Double(0.3193561983429373));

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
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Float(1229.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Float(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Float(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Float(4.364868));

    let a1 = types::TypedValue::Long(1000);
    let a2 = types::TypedValue::Double(229.102);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Double(1229.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Double(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Double(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Double(4.364868050038847));

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
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Float(2899.11112233)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Float(1099.11132233)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Float(1799199.90018858)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Float(2.22123494)
    );

    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::Int(899);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Float(2898.11122233)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Float(1100.11122233)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Float(1797200.98887737)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Float(2.22370548)
    );

    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::Long(899);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Float(2898.11122233)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Float(1100.11122233)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Float(1797200.98887737)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Float(2.22370548)
    );

    let a1 = types::TypedValue::Float(1999.111222333);
    let a2 = types::TypedValue::Double(899.99099);
    // TODO float calculate with double should not loss precision
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Double(2899.1021960546877)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Double(1099.1202160546875)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Double(1799182.0734572522)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Double(2.221256910643836)
    );
}

use common::types;

use bytes::Buf;
use proto::common::common::DataTypeEnum;

#[test]
fn test_typed_value_get_data() {
    let int = types::TypedValue::BigInt(1 << 30);
    let mut data = int.get_data();
    let _ = data.remove(0);
    assert_eq!(data.as_slice().get_i64(), 1 << 30);
    assert_eq!(int.get_type(), DataTypeEnum::DATA_TYPE_ENUM_BIGINT);

    let double = types::TypedValue::Number(1.6546);
    let mut data = double.get_data();
    let _ = data.remove(0);
    assert_eq!(data.as_slice().get_f64(), 1.6546);
    assert_eq!(double.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NUMBER);

    let float = types::TypedValue::Null;
    assert_eq!(float.get_type(), DataTypeEnum::DATA_TYPE_ENUM_NULL);
    let mut data = float.get_data();
    let _ = data.remove(0);
    assert_eq!(data.len(), 0);

    let string = types::TypedValue::String("test".to_string());
    assert_eq!(string.get_type(), DataTypeEnum::DATA_TYPE_ENUM_STRING);
    let mut data = string.get_data();
    let _ = data.remove(0);
    assert_eq!(String::from_utf8(data), Ok("test".to_string()))
}

#[test]
fn test_typed_value_left_int_dual_op() {
    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::BigInt(200);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::BigInt(300));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::BigInt(-100));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::BigInt(20000));
    assert_eq!(a1 / a2, types::TypedValue::BigInt(0));

    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::BigInt(3000);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::BigInt(3100));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::BigInt(-2900));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::BigInt(300000));
    assert_eq!(a1 / a2, types::TypedValue::BigInt(0));

    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::Number(313.129);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Number(413.129));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Number(-213.12900000000002));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Number(31312.9));
    assert_eq!(a1 / a2, types::TypedValue::Number(0.31935719783220334));

    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);

    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);

    let a1 = types::TypedValue::BigInt(100);
    let a2 = types::TypedValue::Invalid;
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1 / a2, types::TypedValue::Invalid);
}

#[test]
fn test_typed_value_left_long_dual_op() {
    std::env::set_var("double.accuracy", 8.to_string());
    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::BigInt(2000);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::BigInt(3000));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::BigInt(-1000));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::BigInt(2000000));
    assert_eq!(a1 / a2, types::TypedValue::BigInt(0));

    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::Number(229.102);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Number(1229.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Number(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Number(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Number(4.364868050038847));

    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::Number(229.102);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Number(1229.102));
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Number(770.898));
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Number(229102.0));
    assert_eq!(a1 / a2, types::TypedValue::Number(4.364868050038847));

    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::BigInt(1000);
    let a2 = types::TypedValue::Invalid;
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);
}

#[test]
fn test_typed_value_left_float_dual_op() {
    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::String("sss".to_string());
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::Boolean(true);
    assert_eq!(a1.clone() + a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() - a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() * a2.clone(), types::TypedValue::Invalid);
    assert_eq!(a1.clone() / a2.clone(), types::TypedValue::Invalid);

    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::Number(899.9999);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Number(2899.111122333)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Number(1099.1113223329999)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Number(1799199.9001885778)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Number(2.2212349382849927)
    );

    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::BigInt(899);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Number(2898.111222333)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Number(1100.111222333)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Number(1797200.988877367)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Number(2.223705475342603)
    );

    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::BigInt(899);
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Number(2898.111222333)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Number(1100.111222333)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Number(1797200.988877367)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Number(2.223705475342603)
    );

    let a1 = types::TypedValue::Number(1999.111222333);
    let a2 = types::TypedValue::Number(899.99099);
    // TODO float calculate with double should not loss precision
    assert_eq!(
        a1.clone() + a2.clone(),
        types::TypedValue::Number(2899.102212333)
    );
    assert_eq!(
        a1.clone() - a2.clone(),
        types::TypedValue::Number(1099.1202323329999)
    );
    assert_eq!(
        a1.clone() * a2.clone(),
        types::TypedValue::Number(1799182.0881075866)
    );
    assert_eq!(
        a1.clone() / a2.clone(),
        types::TypedValue::Number(2.2212569287310306)
    );
}

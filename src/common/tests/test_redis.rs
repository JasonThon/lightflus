use std::collections::BTreeMap;

use bytes::Buf;
use common::{redis::RedisClient, types::TypedValue, utils::get_env};
use proto::common::stream::{RedisDesc, RedisDesc_ConnectionOpts};

#[test]
pub fn test_redis_with_string_key_simple_value() {
    let mut conf = RedisDesc::default();
    let mut opts = RedisDesc_ConnectionOpts::default();
    opts.set_database(0);
    opts.set_host(get_env("REDIS_HOST").unwrap_or("localhost".to_string()));
    conf.set_connection_opts(opts);
    let client = RedisClient::new(&conf);
    let conn_result = client.connect();

    assert!(conn_result.is_ok());
    let conn = &mut conn_result.unwrap();

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::String("value".to_string());

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("value".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::BigInt(123456789);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(result.as_slice().get_i64(), 123456789);

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Number(123456789.123456789);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(result.as_slice().get_f64(), 123456789.123456789);

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Boolean(true);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(result[0], 1);

        let value = &TypedValue::Boolean(false);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");
        assert_eq!(result[0], 0);

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Null;

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("null".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Invalid;

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("undefined".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Array(vec![
            TypedValue::Number(1.2),
            TypedValue::Number(2.3),
            TypedValue::Number(3.4),
        ]);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("[1.2,2.3,3.4]".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Array(vec![
            TypedValue::String("v1".to_string()),
            TypedValue::String("v2".to_string()),
            TypedValue::String("v3".to_string()),
        ]);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(
            String::from_utf8(result),
            Ok("[\"v1\",\"v2\",\"v3\"]".to_string())
        );

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Array(vec![TypedValue::BigInt(1), TypedValue::BigInt(2)]);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("[1,2]".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let value = &TypedValue::Array(vec![TypedValue::Boolean(true), TypedValue::Boolean(false)]);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("[true,false]".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }

    {
        let key = &TypedValue::String("key".to_string());
        let mut val = BTreeMap::default();
        val.insert("k1".to_string(), TypedValue::String("v1".to_string()));
        val.insert("k2".to_string(), TypedValue::Number(123456789.1234567));
        val.insert("k3".to_string(), TypedValue::Boolean(true));
        val.insert("k4".to_string(), TypedValue::BigInt(123456789));
        val.insert(
            "k5".to_string(),
            TypedValue::Array(vec![TypedValue::BigInt(1), TypedValue::BigInt(2)]),
        );
        let value = &TypedValue::Object(val);

        let result = client.set(conn, key, value);
        assert!(result.is_ok());

        let result = client.get(conn, key);
        assert!(result.is_ok());

        let result = result.expect("msg");

        assert_eq!(String::from_utf8(result), Ok("{\"k1\":\"v1\",\"k2\":123456789.1234567,\"k3\":true,\"k4\":123456789,\"k5\":[1,2]}".to_string()));

        let result = client.del(conn, key);
        assert!(result.is_ok());
    }
}

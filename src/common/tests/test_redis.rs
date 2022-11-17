use common::{redis::RedisClient, types::TypedValue, utils::get_env};
use proto::common::stream::{RedisDesc, RedisDesc_ConnectionOpts};

#[test]
pub fn test_redis_with_simple_value() {
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
}

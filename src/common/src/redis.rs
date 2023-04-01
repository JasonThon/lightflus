use std::time::Duration;

use proto::common::RedisDesc;
use redis::{Commands, ConnectionAddr, ConnectionInfo, RedisConnectionInfo, ToRedisArgs};

use crate::{err::RedisException, types::TypedValue};

const REDIS_PORT: u16 = 6379;

#[derive(Debug, Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(conf: &RedisDesc) -> Self {
        let connection_info = to_connection_info(conf);
        let client = redis::Client::open(connection_info);
        Self {
            client: client.expect("create redis client failed"),
        }
    }

    pub fn connect(&self) -> Result<redis::Connection, RedisException> {
        self.client
            .get_connection_with_timeout(Duration::from_secs(3))
            .map_err(|err| RedisException::ConnectFailed(format!("{}", err)))
    }

    pub fn set<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        conn: &mut redis::Connection,
        key: &K,
        value: &V,
    ) -> Result<(), RedisException> {
        conn.set(key, value)
            .map_err(|err| RedisException::SetValueFailed(format!("{}", err)))
    }

    pub fn set_multiple<K: ToRedisArgs, V: ToRedisArgs>(
        &self,
        conn: &mut redis::Connection,
        items: &[(&K, &V)],
    ) -> Result<(), RedisException> {
        conn.set_multiple(items)
            .map_err(|err| RedisException::SetMultipleValueFailed(format!("{}", err)))
    }

    pub fn get<K: ToRedisArgs>(
        &self,
        conn: &mut redis::Connection,
        key: &K,
    ) -> Result<Vec<u8>, RedisException> {
        conn.get(key)
            .map_err(|err| RedisException::GetValueFailed(format!("{}", err)))
    }

    pub fn del<K: ToRedisArgs>(
        &self,
        conn: &mut redis::Connection,
        key: &K,
    ) -> Result<(), RedisException> {
        conn.del(key)
            .map_err(|err| RedisException::DelValueFailed(format!("{}", err)))
    }
}

pub fn to_connection_info(conf: &RedisDesc) -> ConnectionInfo {
    let opts = conf.connection_opts.as_ref().unwrap();
    let db = opts.database;
    let split: Vec<_> = opts.host.split(":").collect();
    let (host, port) = if split.len() < 2 {
        if split.len() == 1 {
            (split[0].to_string(), REDIS_PORT)
        } else {
            ("localhost".to_string(), REDIS_PORT)
        }
    } else {
        (
            split[0].to_string(),
            u16::from_str_radix(split[1], 10).expect("invalid redis port"),
        )
    };
    let (addr, opt) = if opts.tls {
        (
            ConnectionAddr::TcpTls {
                host,
                port,
                insecure: false,
            },
            RedisConnectionInfo {
                db,
                username: if opts.username.is_empty() {
                    Some(opts.username.clone())
                } else {
                    None
                },
                password: Some(opts.password.clone()),
            },
        )
    } else {
        (
            ConnectionAddr::Tcp(host, port),
            RedisConnectionInfo {
                db,
                username: None,
                password: None,
            },
        )
    };
    ConnectionInfo { addr, redis: opt }
}

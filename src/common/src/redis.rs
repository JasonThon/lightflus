use std::time::Duration;

use proto::common::RedisDesc;
use redis::{Commands, ConnectionAddr, ConnectionInfo, RedisConnectionInfo};

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

    pub fn set(
        &self,
        conn: &mut redis::Connection,
        key: &TypedValue,
        value: &TypedValue,
    ) -> Result<(), RedisException> {
        conn.set(key, value)
            .map_err(|err| RedisException::SetValueFailed(format!("{}", err)))
    }

    pub fn set_multiple(
        &self,
        conn: &mut redis::Connection,
        items: &[(&TypedValue, &TypedValue)],
    ) -> Result<(), RedisException> {
        conn.set_multiple(items)
            .map_err(|err| RedisException::SetMultipleValueFailed(format!("{}", err)))
    }

    pub fn get(
        &self,
        conn: &mut redis::Connection,
        key: &TypedValue,
    ) -> Result<Vec<u8>, RedisException> {
        conn.get(key)
            .map_err(|err| RedisException::GetValueFailed(format!("{}", err)))
    }

    pub fn del(
        &self,
        conn: &mut redis::Connection,
        key: &TypedValue,
    ) -> Result<(), RedisException> {
        conn.del(key)
            .map_err(|err| RedisException::DelValueFailed(format!("{}", err)))
    }
}

pub fn to_connection_info(conf: &RedisDesc) -> ConnectionInfo {
    let opts = conf.connection_opts.as_ref().unwrap();
    let db = opts.database;
    let (addr, opt) = if opts.tls {
        (
            ConnectionAddr::TcpTls {
                host: opts.host.clone(),
                port: REDIS_PORT,
                insecure: false,
            },
            RedisConnectionInfo {
                db,
                username: None,
                password: Some(opts.password.clone()),
            },
        )
    } else {
        (
            ConnectionAddr::Tcp(opts.host.clone(), REDIS_PORT),
            RedisConnectionInfo {
                db,
                username: None,
                password: None,
            },
        )
    };
    ConnectionInfo { addr, redis: opt }
}

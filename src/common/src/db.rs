use futures_util::{TryFuture, TryStreamExt};
use prost::Message;
use proto::common::mysql_desc;
use sqlx::{Arguments, ConnectOptions};

use crate::types::TypedValue;

/// Connection of MySQL
///
/// [MysqlConn] can create multiple mysql clients [sqlx::mysql::MySqlConnection].
///
/// A [MysqlConn] instance support two different ways to process the query and its results:
/// - fetch all results in a single set. Such way may blocks current thread in a long duration and consumes a lot of memory.
/// - process elements iteratively
///
/// Lightflus support SQL statment in two different formats:
/// - raw format: e.g. 'select a, b from t';
/// - with placeholder symbol '?': e.g. 'select a, b from t where b=?'
#[derive(Clone)]
pub struct MysqlConn {
    conn_opts: mysql_desc::ConnectionOpts,
}

impl MysqlConn {
    /// # Execute the statement and return the whole result set
    ///
    /// Each SQL statement can be executed by a [MysqlConn] instance with three arguments:
    /// - SQL statement
    /// - List of [TypedValue]. The elements should be aligned with placeholders in the statement.
    /// - Mysql Client [sqlx::mysql::MySqlConnection].
    ///
    /// The return value is a [Result].
    ///
    ///
    /// ## Example
    /// ```
    // / use common::db::MysqlConn;
    // / use proto::common::mysql_desc;
    // / #[tokio::main]
    // / async fn main() {
    // /     let opts = mysql_desc::ConnectionOpts {
    // /         host: "localhost:3306".to_string(),
    // /         username: "root".to_string(),
    // /         password: "pwd".to_string(),
    // /         database: "default"
    // /     };
    // /
    // /     let conn = MysqlConn::from(opts);
    // /     let client = conn.connect().await;
    // /     assert!(client.is_ok());
    // /     let client = client.unwrap();
    // /     let result = client.execute("select * from t", vec![], conn).await;
    // /     assert!(result.is_ok());
    // /}
    /// ```
    pub async fn execute(
        &self,
        statement: &str,
        arguments: Vec<TypedValue>,
        conn: &mut sqlx::mysql::MySqlConnection,
    ) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
        let mut mysql_arg = sqlx::mysql::MySqlArguments::default();
        arguments.iter().for_each(|val| match val {
            TypedValue::String(v) => mysql_arg.add(v),
            TypedValue::BigInt(v) => mysql_arg.add(v),
            TypedValue::Boolean(v) => mysql_arg.add(v),
            TypedValue::Number(v) => mysql_arg.add(v),
            _ => {}
        });

        sqlx::query_with(statement, mysql_arg).execute(conn).await
    }

    /// # TryForEach, processing elements iteratively
    /// The result set of a SQL statement can be processed by a [MysqlConn] instance iteratively with calling method `try_for_each`.
    /// It has four arguments:
    ///
    /// - SQL statement
    /// - List of [TypedValue]. The elements should be aligned with placeholders in the statement.
    /// - Mysql Client [sqlx::mysql::MySqlConnection].
    /// - the processor for each element which will return value implements trait [TryFuture<Ok = (), Error = sqlx::Error>]
    ///
    /// The return value is a [Result]
    ///
    /// # Example
    /// ```
    // / use common::db::MysqlConn;
    // / use proto::common::mysql_desc;
    // / #[tokio::main]
    // / async fn main() {
    // /     let opts = mysql_desc::ConnectionOpts {
    // /         host: "localhost:3306".to_string(),
    // /         username: "root".to_string(),
    // /         password: "pwd".to_string(),
    // /         database: "default"
    // /     };
    // /
    // /     let conn = MysqlConn::from(opts);
    // /     let client = conn.connect().await;
    // /     assert!(client.is_ok());
    // /     let client = client.unwrap();
    // /     let result = client.try_for_each(
    // /             "select name, age from t",
    // /             vec![],
    // /             conn,
    // /             |row| async move {
    // /                 let name = row.try_get::<&str, &str>("name");
    // /                 assert!(name.is_ok());
    // /                 let age = row.try_get::<i32, &str>("age");
    // /                 assert!(age.is_ok());
    // /             }
    // /     ).await;
    // /     assert!(result.is_ok());
    // /}
    /// ```
    pub async fn try_for_each<
        Fut: TryFuture<Ok = (), Error = sqlx::Error>,
        F: FnMut(sqlx::mysql::MySqlRow) -> Fut,
    >(
        &self,
        statement: &str,
        arguments: Vec<TypedValue>,
        conn: &mut sqlx::mysql::MySqlConnection,
        mut f: F,
    ) -> Result<(), sqlx::Error> {
        let mut mysql_arg = sqlx::mysql::MySqlArguments::default();
        arguments.iter().for_each(|val| match val {
            TypedValue::String(v) => mysql_arg.add(v),
            TypedValue::BigInt(v) => mysql_arg.add(v),
            TypedValue::Boolean(v) => mysql_arg.add(v),
            TypedValue::Number(v) => mysql_arg.add(v),
            _ => {}
        });

        sqlx::query_with(statement, mysql_arg)
            .fetch(conn)
            .try_for_each(|row| f(row))
            .await
    }

    pub async fn connect(&self) -> Result<sqlx::mysql::MySqlConnection, sqlx::Error> {
        let opts = sqlx::mysql::MySqlConnectOptions::new()
            .host(&self.conn_opts.host)
            .port(3306)
            .username(&self.conn_opts.username)
            .password(&self.conn_opts.password)
            .database(&self.conn_opts.database);

        opts.connect().await
    }

    pub fn close(&mut self) {
        self.conn_opts.clear()
    }
}

impl From<mysql_desc::ConnectionOpts> for MysqlConn {
    fn from(conn_opts: mysql_desc::ConnectionOpts) -> Self {
        Self { conn_opts }
    }
}

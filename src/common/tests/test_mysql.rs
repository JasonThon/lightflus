use common::db::MysqlConn;
use proto::common::mysql_desc;

#[tokio::test]
async fn test_mysql_connected() {
    let conn_opts = mysql_desc::ConnectionOpts {
        host: "localhost".to_string(),
        username: "ci".to_string(),
        password: "123".to_string(),
        database: "ci".to_string(),
    };

    let conn = MysqlConn::from(conn_opts);

    let result = conn.connect().await;
    assert!(result.is_ok())
}

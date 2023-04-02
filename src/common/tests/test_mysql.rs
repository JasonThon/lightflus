use common::{db::MysqlConn, types::TypedValue};
use proto::common::mysql_desc;
use sqlx::Row;

#[tokio::test]
async fn test_mysql_execute() {
    let conn_opts = mysql_desc::ConnectionOpts {
        host: "localhost".to_string(),
        username: "ci".to_string(),
        password: "123".to_string(),
        database: "ci".to_string(),
    };

    let mut conn = MysqlConn::from(conn_opts);

    let result = conn.execute("create table if not exists person (id int NOT NULL AUTO_INCREMENT, name varchar(36), age int, country varchar(36), address varchar(255), PRIMARY KEY (id))", vec![]).await;
    assert!(result.is_ok());

    let result = conn.execute("insert into person (name,age,country,address) values ('jason thon',25,'China','Songjiang,Shanghai')", vec![]).await;
    assert!(result.is_ok());

    let result = conn
        .try_for_each(
            "select * from person where name=? and age=?",
            vec![
                TypedValue::String("jason thon".to_string()),
                TypedValue::BigInt(25),
            ],
            |row| async move {
                let name = row.try_get::<&str, &str>("name");
                assert_eq!(name.unwrap(), "jason thon");
                let age = row.try_get::<i32, &str>("age");
                assert_eq!(age.unwrap(), 25);
                let country = row.try_get::<&str, &str>("country");
                assert_eq!(country.unwrap(), "China");
                let address = row.try_get::<&str, &str>("address");
                assert_eq!(address.unwrap(), "Songjiang,Shanghai");
                Ok(())
            },
        )
        .await;

    assert!(result.is_ok());

    let result = conn
        .try_for_each("select * from person", vec![], |row| async move {
            let name = row.try_get::<&str, &str>("name");
            assert_eq!(name.unwrap(), "jason thon");
            let age = row.try_get::<i32, &str>("age");
            assert_eq!(age.unwrap(), 25);
            let country = row.try_get::<&str, &str>("country");
            assert_eq!(country.unwrap(), "China");
            let address = row.try_get::<&str, &str>("address");
            assert_eq!(address.unwrap(), "Songjiang,Shanghai");
            Ok(())
        })
        .await;

    assert!(result.is_ok());

    let result = conn.execute("drop table if exists person", vec![]).await;
    assert!(result.is_ok());
}

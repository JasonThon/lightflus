use std::collections;

use actix_web::dev::Service;
use mongodb::bson::doc;
use mongodb::Database;

use crate::{api, core::domain, core::err};

const TABLE_ROW_RECORD: &str = "table.row.record";
const TABLE_COL_RECORD: &str = "table.column.record";
const TABLE: &str = "table";

pub enum TableRepo {
    Mongo(mongodb::Database)
}

impl<'a> TableRepo {
    pub async fn do_action(&self, action: TableRepoAction<'a>) -> Result<(), err::TableError> {
        match action {
            TableRepoAction::InsertSingle(record) =>
                match self {
                    TableRepo::Mongo(db) => {
                        match self.check_table_exist(db, record.id()) {
                            Ok(table) => match db.collection::<domain::TableRowRecord>(TABLE_ROW_RECORD)
                                .insert_one(record, None)
                                .await
                                .map_err(|err| {
                                    err::TableError::InsertRecordFailed(err)
                                }) {
                                Ok(_) => self.log_new_column_update(db, &record.to_column()).await,
                                Err(err) => Err(err)
                            },
                            Err(err) => Err(err)
                        }
                    }
                }

            TableRepoAction::Create(schema) =>
                match self {
                    TableRepo::Mongo(db) =>
                        db.collection::<domain::TableSchema>(TABLE)
                            .insert_one(schema, None)
                            .await
                            .map_err(|err| err::TableError::InsertRecordFailed(err))
                            .map(|r| ())
                }
        }
    }

    async fn log_new_column_update(&self, db: &Database, records: &Vec<domain::TableColumnRecord>) -> Result<(), err::TableError> {
        db.collection::<domain::TableColumnUpdateLog>(TABLE_ROW_RECORD)
            .insert_one(domain::TableColumnUpdateLog::new(records.to_vec()), None)
            .await
            .map_err(|err| err::TableError::InsertRecordFailed(err))
            .map(|r| ())
    }

    fn check_table_exist(&self, db: &Database, table_id: String) -> Result<domain::TableSchema, err::TableError> {
        db.collection::<domain::TableSchema>(TABLE)
            .find_one(doc! {"id": table_id.as_str()}, None)
            .await
            .map_err(|err| err::TableError::FailToFindDocument(err))
            .and_then(|option| option.ok_or(err::TableError::TableNotExist(table_id)))
    }
}

pub enum TableRepoAction<'a> {
    InsertSingle(&'a domain::TableRowRecord),
    Create(&'a domain::TableSchema),
}
use std::collections::BTreeMap;
use mongodb::bson::doc;

use tokio::sync::mpsc;

use core::event;

use crate::api;
use crate::api::TableResponse;
use crate::core::{domain, repo};
use crate::core::domain::TableSchema;
use crate::core::err;

pub struct TableWorker {
    tx: mpsc::UnboundedSender<event::Event>,
    repo: repo::TableRepo,
}

impl TableWorker {
    pub async fn new(desc: core::mongo::MongoDesc,
                     tx: mpsc::UnboundedSender<event::Event>) -> TableWorker {
        TableWorker {
            tx,
            repo: repo::TableRepo::Mongo(desc.to_db().await?),
        }
    }
}

impl TableWorker {
    pub async fn process_cmd(&self, cmd: api::TableCommand) -> Result<core::http::Response, err::TableError> {
        match cmd.action() {
            api::TableAction::Insert {
                data, id
            } => {
                match domain::new_row_record(data, id) {
                    Ok(record) => {
                        self.repo.do_action(
                            repo::TableRepoAction::InsertSingle(&record))
                            .await
                            .and_then(|_| self.tx.send(
                                event::Event::new(
                                    event::EventType::TableEvent(
                                        event::table::Action::Insert {
                                            data: record.data().clone(),
                                            id: record.id(),
                                        })
                                ))
                                .map_err(|err| err::TableError::EventSendError(err))
                            )
                            .map(|_| core::http::Response::ok())
                    },
                    Err(err) => Err(err)
                }
            }

            api::TableAction::Create { name , _type} => {
                Ok(core::http::Response::ok())
            }
        }
    }
}
use std::{collections, sync};

use mongodb::bson::doc;
use tokio::sync::mpsc;

use crate::cluster;
use common::{err, event, types, types::stream};

pub const COORD_JOB_GRAPH_COLLECTION: &str = "coord.job.graph";

pub enum JobStorage {
    Mongo(mongodb::sync::Collection<types::DataflowContext>),
}

impl JobStorage {
    pub fn find_one(&self,
                    table_id: &str,
                    header_id: &str) -> Result<Option<types::DataflowContext>, err::CommonException> {
        match self {
            JobStorage::Mongo(mongo) => mongo.find_one(
                doc! {
                        "jobId.headerId": header_id,
                        "jobId.tableId": table_id
                    },
                None)
                .map_err(|err| err::CommonException::from(err))
        }
    }

    pub fn find_all(&self) -> Result<Vec<types::DataflowContext>, err::CommonException> {
        match self {
            JobStorage::Mongo(mongo) => mongo.find(None, None)
                .map_err(|err| err::CommonException::from(err))
                .map(|cursor|
                    cursor.filter_map(|result| result.ok())
                        .collect()
                )
        }
    }

    pub fn create(&self, graph: &types::DataflowContext) -> Result<(), err::CommonException> {
        match self {
            JobStorage::Mongo(mongo) => match mongo.insert_one(
                graph,
                None,
            ) {
                Err(err) => {
                    if common::mongo::is_dup_err(&err) {
                        return match mongo.replace_one(
                            doc! {
                            "jobId.headerId": graph.job_id.header_id.as_str(),
                            "jobId.tableId": graph.job_id.table_id.as_str()
                        },
                            graph,
                            None,
                        ) {
                            Ok(_) => Ok(()),
                            Err(err) => Err(err.into())
                        };
                    }

                    Err(err.into())
                }
                _ => Ok(())
            }
        }
    }
}

pub struct Coordinator {
    job_storage: JobStorage,
    connector_proxy: String,
}

impl Coordinator {
    pub fn new(job_storage: JobStorage,
               connector_proxy: String) -> Self {
        Coordinator {
            job_storage,
            connector_proxy,
        }
    }

    pub fn init(&self) -> Result<Vec<types::DataflowContext>, err::CommonException> {
        self.job_storage.find_all()
            .map(|models| {
                common::lists::for_each(
                    &models,
                    |model| send_to_connector(
                        model,
                        BindAction::Create,
                        &self.connector_proxy,
                    ),
                );
                models
            })
    }

    pub fn submit_job(&self,
                      table_id: &String,
                      header_id: &String,
                      graph: &stream::StreamGraph,
                      cluster: sync::RwLockReadGuard<cluster::Cluster>) -> Result<(), err::CommonException> {
        let ref job_id = types::job_id(table_id.as_str(), header_id.as_str());
        if !cluster.is_available() {
            return Err(err::CommonException::new(err::ErrorKind::NoAvailableWorker, "no available worker"));
        }
        let ref context = types::DataflowContext::new(
            job_id.clone(),
            graph.meta.to_vec(),
            types::NodeSet::from_iter(graph.data.iter()
                .map(|(id, value)|
                    (id.clone(), types::OperatorInfo {
                        addr: cluster.partition_key(value).unwrap(),
                        value: value.clone(),
                        id: id.parse::<u64>().unwrap(),
                        upstream: graph.find_upstreams(id.parse::<u64>().unwrap()),
                    })
                )
            ));

        match self.job_storage.find_one(
            job_id.table_id.as_str(),
            job_id.header_id.as_str(),
        ) {
            Ok(option) => {
                match option {
                    Some(graph) => {
                        match cluster
                            .stop_job(job_id)
                            .map(|_| send_to_connector(&graph, BindAction::Stop, &self.connector_proxy))
                            .map_err(|err| err.into()) {
                            Err(err) => return Err(err),
                            Ok(_) => {}
                        }
                    }
                    None => {}
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }


        self.job_storage.create(context)
            .and_then(|_| context.dispatch())
            .map(|_| send_to_connector(
                context,
                BindAction::Create,
                &self.connector_proxy)
            )
    }
}

enum BindAction {
    Create,
    Stop,
}

fn send_to_connector(graph: &types::DataflowContext,
                     binder_action: BindAction,
                     connector_proxy: &String) {
    let mut binder_events = vec![];
    for (_, operator) in &graph.nodes {
        match &operator.value {
            stream::OperatorType::Reference { table_id, header_id, .. } => {
                let binder_type = match binder_action {
                    BindAction::Create => event::BinderEventType::Create {
                        table_id: table_id.clone(),
                        header_id: header_id.clone(),
                        id: operator.id,
                        addr: operator.addr.clone(),
                    },
                    BindAction::Stop => event::BinderEventType::Stop,
                };

                binder_events.push(event::BinderEvent {
                    job_id: graph.job_id.clone(),
                    binder_type,
                })
            }
            _ => {}
        }
    }

    match serde_json::to_vec(&binder_events) {
        Ok(data) => {
            let ref mut request = dataflow_api::probe::EventRequest::new();
            request.set_data(data);
            match dataflow_api::connector::new_connector_client(connector_proxy)
                .handle_event(request) {
                Err(err) => log::error!("failed to update binder {}", err),
                _ => {}
            }
        }
        Err(err) => log::error!("fail to serialize events {}", err)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub mongo: common::mongo::MongoConfig,
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub conn_proxy: String,
}
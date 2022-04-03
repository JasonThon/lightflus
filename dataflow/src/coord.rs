use std::{collections, sync};
use mongodb::bson::doc;
use tokio::sync::mpsc;

use crate::{cluster, err, event, types};
use crate::types::formula;

pub const COORD_JOB_GRAPH_COLLECTION: &str = "coord.job.graph";

pub enum JobRepo {
    Mongo(mongodb::sync::Collection<types::GraphModel>),
}

impl JobRepo {
    pub fn find_one(&self,
                    table_id: &str,
                    header_id: &str) -> Result<Option<types::GraphModel>, err::CommonException> {
        match self {
            JobRepo::Mongo(mongo) => mongo.find_one(
                doc! {
                        "jobId.headerId": header_id,
                        "jobId.tableId": table_id
                    },
                None)
                .map_err(|err| err::CommonException::from(err))
        }
    }

    pub fn create(&self, graph: &types::GraphModel) -> Result<(), err::CommonException> {
        match self {
            JobRepo::Mongo(mongo) => match mongo.insert_one(
                graph,
                None,
            ) {
                Err(err) => {
                    if core::mongo::is_dup_err(&err) {
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
    job_repo: JobRepo,
    senders: Vec<mpsc::UnboundedSender<Vec<event::BinderEvent>>>,
}

impl Coordinator {
    pub fn new(job_repo: JobRepo,
               senders: Vec<mpsc::UnboundedSender<Vec<event::BinderEvent>>>) -> Self {
        Coordinator {
            job_repo,
            senders,
        }
    }

    pub fn submit_job(&self,
                      table_id: &String,
                      header_id: &String,
                      graph: &formula::FormulaGraph,
                      cluster: sync::RwLockReadGuard<cluster::Cluster>) -> Result<(), err::CommonException> {
        let ref job_id = types::job_id(table_id.as_str(), header_id.as_str());
        let ref execution_graph = types::GraphModel::new(
            job_id.clone(),
            graph.meta.to_vec(),
            collections::BTreeMap::from_iter(graph.data.iter()
                .map(|(id, value)|
                    (id.clone(), types::Operator {
                        addr: cluster.partition_key(value),
                        value: value.clone(),
                        id: id.clone(),
                    })
                )
            ));

        match self.job_repo.find_one(
            job_id.table_id.as_str(),
            job_id.header_id.as_str(),
        ) {
            Ok(option) => {
                match option {
                    Some(graph) => {
                        match cluster
                            .stop_job(job_id)
                            .map(|_| send_to_conns(&graph, BindAction::STOP, &self.senders))
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


        self.job_repo.create(execution_graph)
            .and_then(|_| execution_graph.dispatch())
            .map(|_| send_to_conns(
                execution_graph,
                BindAction::CREATE,
                &self.senders)
            )
    }
}

enum BindAction {
    CREATE,
    STOP,
}

fn send_to_conns(graph: &types::GraphModel,
                 binder_action: BindAction,
                 senders: &Vec<mpsc::UnboundedSender<Vec<event::BinderEvent>>>) {
    let mut binder_events = vec![];
    for (_, operator) in &graph.nodes {
        match &operator.value {
            formula::FormulaOp::Reference { table_id, header_id } => {
                let binder_type = match binder_action {
                    BindAction::CREATE => event::BinderEventType::Create {
                        table_id: table_id.clone(),
                        header_id: header_id.clone(),
                        id: operator.id,
                        addr: operator.addr.clone(),
                    },
                    BindAction::STOP => event::BinderEventType::Stop {},
                };

                binder_events.push(event::BinderEvent {
                    job_id: graph.job_id.clone(),
                    binder_type,
                })
            }
            _ => {}
        }
    }

    core::lists::for_each(senders, |sender| {
        let _ = sender.send(binder_events.clone());
    })
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CoordinatorConfig {
    pub mongo: core::mongo::MongoConfig,
    pub port: usize,
    pub cluster: Vec<cluster::NodeConfig>,
    pub sources: Vec<types::SourceDesc>,
}
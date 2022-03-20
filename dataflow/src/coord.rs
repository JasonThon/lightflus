use std::cell;
use std::collections;

use tokio::sync::mpsc;

use crate::{cluster, runtime, types};
use crate::err;
use crate::err::CoordinatorException;
use crate::runtime::execution;
use crate::types::formula;

pub struct Coordinator {
    cluster: cluster::ClusterConfig,
    job_map: cell::RefCell<collections::BTreeMap<types::JobID, runtime::Graph>>,
}

impl Coordinator {
    pub fn new() -> Self {
        Coordinator {
            cluster: cluster::ClusterConfig::new(),
            job_map: cell::RefCell::new(Default::default()),
        }
    }

    pub fn submit_job(&self,
                      table_id: &String,
                      header_id: &String,
                      graph: &formula::FormulaGraph) -> Result<(), err::CoordinatorException> {
        let ref job_id = self.job_id(table_id, header_id);

        let mut map = self.job_map.take();

        match map
            .insert(
                job_id.clone(),
                runtime::Graph::new(
                    job_id.clone(),
                    graph.meta.to_vec(),
                    collections::BTreeMap::from_iter(graph.data.iter()
                        .map(|(id, value)|
                            (id.clone(), execution::Operator {
                                addr: self.cluster.partition_key(value).to_string(),
                                value: value.clone(),
                                id: id.clone(),
                            })
                        )
                    ),
                    false
                ),
            ) {
            Some(graph) => return graph.remove_nodes()
                .map_err(|err| CoordinatorException::from(err)),

            None => {}
        }

        self.job_map.replace(map);

        self.job_map.borrow()
            .get(job_id)
            .unwrap()
            .dispatch()
            .map_err(|err| CoordinatorException::from(err))
    }
    fn job_id(&self, table_id: &String, header_id: &String) -> types::JobID {
        todo!()
    }
}

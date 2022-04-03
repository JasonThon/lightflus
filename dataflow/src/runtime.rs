use crate::types;

pub mod execution {
    use actix::Actor;
    use serde::ser::SerializeStruct;
    use crate::{err, event, types};
    use crate::types::formula;

    pub struct ExecutionGraph {
        pub job_id: types::JobID,
        pub meta: types::AdjacentList,
        pub nodes: types::NodeSet,
        addrmap: types::AddrMap,
    }

    impl super::Graph {
        pub fn new(job_id: types::JobID,
                   meta: types::AdjacentList,
                   nodes: types::NodeSet) -> Self {
            ExecutionGraph {
                job_id,
                meta,
                nodes,
                addrmap: Default::default(),
            }
        }

        pub fn build_dag(&mut self, job_id: types::JobID) {
            self.addrmap = build_graph(job_id, &self.meta, &self.nodes);
        }

        pub fn try_recv(&self, event: event::FormulaOpEvent) -> Result<(), err::ExecutionException> {
            match self.addrmap
                .get(&event.to) {
                Some(addr) => addr
                    .do_send(event)
                    .map_err(|err|
                        err::ExecutionException {
                            kind: err::ErrorKind::ActorSendError,
                            msg: err.to_string(),
                        }
                    ),
                None => Ok(())
            }
        }

        pub fn stop(&self) -> Result<(), err::ExecutionException> {
            for (id, addr) in &self.addrmap {
                match addr.do_send(event::FormulaOpEvent {
                    job_id: self.job_id.clone(),
                    from: 0,
                    to: id.clone(),
                    event_type: event::FormulaOpEventType::Stop,
                    data: vec![],
                    event_time: std::time::SystemTime::now(),
                }) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("send event failed: {:?}", &err);
                        return Err(err::ExecutionException::fail_send_event_to_job_graph(&self.job_id));
                    }
                }
            }

            Ok(())
        }
    }

    impl serde::Serialize for super::Graph {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
            let mut result = serializer.serialize_struct("Graph", 2)?;
            let _ = result.serialize_field("jobId", &self.job_id);
            let _ = result.serialize_field("nodes", &self.nodes);
            let _ = result.serialize_field("meta", &self.meta);
            result.end()
        }
    }

    pub(crate) enum Node {
        Local {
            op: formula::FormulaOp,
            id: u64,
            recipients: Vec<actix::Recipient<event::FormulaOpEvent>>,
        },
        Ref {
            addr: String,
            id: u64,
            job_id: types::JobID,
        },
        None,
    }

    impl Default for Node {
        fn default() -> Self {
            Node::None
        }
    }

    impl actix::Actor for Node {
        type Context = actix::Context<Self>;
    }

    impl actix::Handler<event::FormulaOpEvent> for Node {
        type Result = ();

        fn handle(&mut self, msg: event::FormulaOpEvent, ctx: &mut Self::Context) -> Self::Result {
            todo!()
        }
    }

    fn to_node(op: &types::Operator,
               job_id: types::JobID,
               local_hostname: &String,
               recipients: Vec<actix::Recipient<event::FormulaOpEvent>>) -> Node {
        if op.addr.starts_with(local_hostname) {
            return Node::Local {
                op: op.value.clone(),
                id: op.id.clone(),
                recipients,
            };
        }

        Node::Ref {
            addr: op.addr.clone(),
            id: op.id.clone(),
            job_id,
        }
    }

    pub(crate) fn build_graph(job_id: types::JobID,
                              meta: &types::AdjacentList,
                              nodes: &types::NodeSet) -> types::AddrMap {
        let ref local_hostname = core::hostname().expect("");

        let traversed = types::traverse_from_bottom(meta);
        let mut addrmap = types::AddrMap::new();

        for ref adj in traversed {
            let recipients = core::lists::map(
                &adj.neighbors,
                |neighbor_id| {
                    match addrmap.get(neighbor_id) {
                        Some(addr) => (*addr).clone(),
                        None => {
                            let operator = nodes
                                .get(neighbor_id)
                                .unwrap();

                            addrmap.insert(
                                neighbor_id.clone(),
                                to_node(operator, job_id.clone(), local_hostname, vec![])
                                    .start()
                                    .recipient(),
                            );

                            (*addrmap
                                .get(neighbor_id)
                                .unwrap()
                            ).clone()
                        }
                    }
                });

            addrmap.insert(
                adj.center.clone(),
                to_node(nodes.get(&adj.center)
                            .unwrap(),
                        job_id.clone(),
                        local_hostname,
                        recipients)
                    .start()
                    .recipient(),
            );
        }

        addrmap
    }
}

pub type Graph = execution::ExecutionGraph;

pub fn to_execution_graph(model: types::GraphModel) -> Graph {
    Graph::new(
        model.job_id,
        model.meta,
        model.nodes,
    )
}


pub mod formula {
    use crate::types::formula::FormulaOp;

    impl FormulaOp {}
}

pub fn from_str(value: &str) -> Result<Graph, serde_json::Error> {
    let result = serde_json::from_str::<types::GraphModel>(value);

    result.map(|model| to_execution_graph(model))
}
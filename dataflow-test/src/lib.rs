mod utils {
    use std::collections;
    use dataflow::types;

    pub fn default_adj_vec() -> Vec<types::AdjacentVec> {
        vec![
            types::AdjacentVec {
                neighbors: vec![1, 2, 3],
                center: 0,
            },
            types::AdjacentVec {
                neighbors: vec![4, 5],
                center: 1,
            },
            types::AdjacentVec {
                neighbors: vec![5, 6],
                center: 2,
            },
            types::AdjacentVec {
                neighbors: vec![7],
                center: 3,
            },
            types::AdjacentVec {
                neighbors: vec![8],
                center: 6,
            },
        ]
    }

    pub fn default_formula_graph() -> types::formula::FormulaGraph {
        let mut values = vec![];

        for (id, operator) in default_nodeset().iter() {
            values.push((id.clone(), operator.value.clone()))
        }

        types::formula::FormulaGraph {
            meta: default_adj_vec(),
            data: collections::BTreeMap::from_iter(values),
        }
    }

    pub fn default_nodeset() -> types::NodeSet {
        types::NodeSet::from(
            [
                ("0".to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Reference {
                        table_id: "tableId_1".to_string(),
                        header_id: "headerId_1".to_string(),
                    },
                    id: 0,
                }),
                ("1".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 1,
                }),
                ("2".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 2,
                }),
                ("3".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 3,
                }),
                ("4".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 4,
                }),
                ("5".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 5,
                }),
                ("6".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 6,
                }),
                ("7".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 6,
                }),
                ("8".to_string(), types::Operator {
                    addr: "".to_string(),
                    value: types::formula::FormulaOp::Add,
                    id: 8,
                }),
            ]
        )
    }
}


#[cfg(test)]
mod runtime_test {
    use dataflow::{runtime, types};

    fn default_graph() -> runtime::Graph {
        runtime::Graph::new(
            types::job_id("tableId", "headerId"),
            super::utils::default_adj_vec(),
            super::utils::default_nodeset(),
        )
    }

    #[test]
    fn test_graph_serialize() {
        let ref graph = default_graph();

        let json = serde_json::to_string(graph);

        assert!(json.is_ok());
        println!("{}", json.unwrap())
    }

    #[test]
    fn test_graph_deserialize() {
        let ref graph = default_graph();
        let json = serde_json::to_string(graph);
        assert!(json.is_ok());
        println!("{}", json.as_ref().unwrap());

        let result = runtime::from_str(json.unwrap().as_str());
        if result.is_err() {
            panic!("{:?}", result.unwrap_err())
        }
        assert!(result.is_ok());

        let target_graph = result.unwrap();
        assert_eq!(graph.job_id, target_graph.job_id);
        assert_eq!(graph.meta, target_graph.meta);
    }


    #[actix::test]
    async fn test_addr_map() {
        let addr_map = runtime::execution::build_addr_map(
            types::job_id("tableId", "headerId"),
            &vec![
                types::AdjacentVec {
                    neighbors: vec![1, 2, 3],
                    center: 0,
                },
                types::AdjacentVec {
                    neighbors: vec![4, 5],
                    center: 1,
                },
                types::AdjacentVec {
                    neighbors: vec![5, 6],
                    center: 2,
                },
                types::AdjacentVec {
                    neighbors: vec![7],
                    center: 3,
                },
                types::AdjacentVec {
                    neighbors: vec![8],
                    center: 6,
                },
                types::AdjacentVec {
                    neighbors: vec![6],
                    center: 7,
                },
            ],
            &super::utils::default_nodeset(),
        );

        for id in 0..8 {
            assert!(addr_map.contains_key(&(id as u64)));
        }
    }
}

#[cfg(test)]
mod worker_test {
    use actix::Actor;
    use dataflow::event;
    use dataflow::worker;

    #[test]
    fn test_worker_handler() {
        let runner = actix::System::new();
        runner.block_on(async {
            let worker = worker::new_worker();
            let addr = worker.start();
            addr.recipient().do_send(event::GraphEvent::StopGraph { job_id: Default::default() });
        });

        runner.run();
    }
}

#[cfg(test)]
mod types_test {
    use dataflow::{event, types};

    fn default_empty_graph() -> types::GraphModel {
        types::GraphModel {
            job_id: types::job_id("tableId", "headerId"),
            meta: vec![],
            nodes: Default::default(),
        }
    }

    #[test]
    fn test_event_serialize() {
        let submit = event::GraphEvent::NodeEventSubmit(event::FormulaOpEvent {
            job_id: types::job_id("tableId", "headerId"),
            from: 0,
            to: 0,
            event_type: event::FormulaOpEventType::Insert,
            data: vec![types::Entry { row_idx: 0, value: vec![1, 2, 3] }],
            event_time: std::time::SystemTime::now(),
        });
        let result = serde_json::to_string(&submit);
        assert!(result.is_ok());
        let string = result.unwrap();

        let json = serde_json::from_str::<event::GraphEvent>(string.as_str());
        if json.is_err() {
            panic!("{}",json.unwrap_err());
        }

        assert!(json.is_ok());
    }

    #[test]
    fn test_serialize_empty_graph() {
        let ref model = default_empty_graph();
        let result = serde_json::to_string(model);
        assert!(result.is_ok());

        let value = result.unwrap();
        let deserialize_model = serde_json::from_str::<types::GraphModel>(value.as_str());
        assert!(deserialize_model.is_ok());

        let ref deser_model = deserialize_model.unwrap();
        assert_eq!(&deser_model.job_id, &model.job_id);
        assert!((&deser_model.nodes).is_empty());
        assert!((&deser_model.meta).is_empty());
    }

    #[test]
    fn test_traverse_from_bottom() {
        let ref meta = vec![
            types::AdjacentVec {
                center: 4,
                neighbors: vec![8, 9],
            },
            types::AdjacentVec {
                center: 0,
                neighbors: vec![1, 2, 3],
            },
            types::AdjacentVec {
                center: 1,
                neighbors: vec![4, 5],
            },
            types::AdjacentVec {
                center: 3,
                neighbors: vec![5, 7],
            },
        ];

        let traversed = types::traverse_from_bottom(meta);

        assert_eq!(traversed, vec![
            types::AdjacentVec {
                center: 4,
                neighbors: vec![8, 9],
            },
            types::AdjacentVec {
                center: 1,
                neighbors: vec![4, 5],
            },
            types::AdjacentVec {
                center: 3,
                neighbors: vec![5, 7],
            },
            types::AdjacentVec {
                center: 0,
                neighbors: vec![1, 2, 3],
            },
        ])
    }

    #[test]
    fn test_job_id_eq() {
        let id = types::job_id("tableId", "headerId");
        let id1 = types::job_id("tableId", "headerId");
        assert_eq!(id, id1);
    }

    #[test]
    fn test_serialize_graph_event() {
        let ref graph_event = event::TableEvent::new(
            event::TableAction::FormulaUpdate {
                table_id: "tableId".to_string(),
                header_id: "headerId".to_string(),
                graph: super::utils::default_formula_graph(),
            }
        );

        let result = serde_json::to_string(graph_event);
        assert!(result
            .as_ref()
            .map(|value| {
                println!("{:?}", value);
                println!("{:?}", value.as_bytes())
            }).is_ok());
        let value = result.unwrap();

        let event_r = serde_json::from_slice::<event::TableEvent>(value.as_bytes());
        if event_r.is_err() {
            panic!("{}", event_r.unwrap_err())
        }
        assert!(event_r.is_ok());
    }
}

#[cfg(test)]
mod conn_test {
    use tokio::sync::mpsc;
    use dataflow::{event, types};

    #[tokio::test]
    async fn test_disconnect() {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (mut disconnect_tx, disconnect_rx) = mpsc::channel(1);

        let ref desc = types::SourceDesc::Tableflow {
            host: "localhost".to_string(),
            port: 0,
            limit: 10,
            event_time: None,
        };

        let connector = dataflow::conn::Connector::new(desc, event_rx, disconnect_rx);
        let handle = tokio::spawn(connector.start());

        let result = disconnect_tx.send(event::Disconnect).await;
        assert!(result.is_ok());

        let r = handle.await;
        assert!(r.is_ok());

        let redisconnect = disconnect_tx.send(event::Disconnect).await;
        assert!(redisconnect.is_err());
        assert!(event_tx.send(vec![]).is_err());
    }
}

#[cfg(test)]
mod event_test {
    use dataflow::event;

    #[test]
    fn test_connect_event_serialize() {
        let ref connector_event = event::ConnectorEvent {
            event_type: event::ConnectorEventType::Action(event::INSERT),
            table_id: "tableId".to_string(),
            header_id: "headerId".to_string(),
            entries: vec![],
            timestamp: "2022-04-06 11:52:50".to_string(),
        };

        let result = serde_json::to_string(connector_event);
        assert!(result.is_ok());
        let ref value = result.unwrap();
        println!("{:?}", value);

        let json = serde_json::from_str::<event::ConnectorEvent>(value.as_str());
        assert!(json.is_ok());

        let deser_event = json.unwrap();
        assert_eq!(&deser_event.event_type, &connector_event.event_type);
        assert_eq!(&deser_event.table_id, &connector_event.table_id);
        assert_eq!(&deser_event.header_id, &connector_event.header_id);
        assert_eq!(&deser_event.timestamp, &connector_event.timestamp);
    }
}
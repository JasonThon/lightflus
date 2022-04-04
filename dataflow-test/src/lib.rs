#[cfg(test)]
mod runtime_test {
    use std::collections;

    use dataflow::{runtime, types};

    fn default_graph() -> runtime::Graph {
        runtime::Graph::new(
            types::job_id("tableId", "headerId"),
            vec![
                types::AdjacentVec {
                    center: 0,
                    neighbors: vec![1, 2],
                }
            ],
            collections::BTreeMap::from(
                [
                    (0, types::Operator { addr: "localhost".to_string(), id: 0, value: types::formula::FormulaOp::Add }),
                    (1, types::Operator {
                        addr: "localhost".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 1,
                    }),
                    (2, types::Operator {
                        addr: "localhost".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 2,
                    })
                ]
            ))
    }

    #[test]
    fn test_graph_serialize() {
        let ref graph = default_graph();

        let json = serde_json::to_string(graph);

        assert!(json.is_ok());
    }

    #[test]
    fn test_graph_deserialize() {
        let ref graph = default_graph();
        let json = serde_json::to_string(graph);
        assert!(json.is_ok());

        let result = runtime::from_str(json.unwrap().as_str());
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
            &types::NodeSet::from(
                [
                    (0, types::Operator {
                        addr: "localhost".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 0,
                    }),
                    (1, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 1,
                    }),
                    (2, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 2,
                    }),
                    (3, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 3,
                    }),
                    (4, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 4,
                    }),
                    (5, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 5,
                    }),
                    (6, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 6,
                    }),
                    (7, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 6,
                    }),
                    (8, types::Operator {
                        addr: "".to_string(),
                        value: types::formula::FormulaOp::Add,
                        id: 8,
                    }),
                ]
            ),
        );

        for id in 0..8 {
            assert!(addr_map.contains_key(&(id as u64)));
        }
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
    fn test_serialize_graph_event() {
        let ref graph_event = event::TableEvent::new(
            event::TableAction::FormulaUpdate {
                table_id: "tableId".to_string(),
                header_id: "headerId".to_string(),
                graph: types::formula::FormulaGraph { meta: vec![], data: Default::default() },
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
        assert!(event_r.is_ok());
    }
}

#[cfg(test)]
mod conn_test {
    use std::time;

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
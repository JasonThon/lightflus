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
}

#[cfg(test)]
mod types_test {
    use dataflow::{event, types};

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
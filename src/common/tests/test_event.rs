use common::{event, types};
use common::event::{Event, GraphEvent};
use utils::*;

mod utils;

#[test]
fn test_event_serialize() {
    let submit = event::GraphEvent::DataSourceEventSubmit(event::DataSourceEvent {
        job_id: types::job_id("tableId", "headerId"),
        to: 0,
        event_type: types::DataSourceEventType::Insert,
        data: vec![types::Entry { row_idx: 0, value: vec![1, 2, 3] }],
        old_data: vec![],
        event_time: std::time::SystemTime::now(),
    });
    let result = serde_json::to_string(&submit);
    assert!(result.is_ok());
    let string = result.unwrap();

    let json = serde_json::from_str::<event::GraphEvent>(string.as_str());
    if json.is_err() {
        panic!("{}", json.unwrap_err());
    }

    assert!(json.is_ok());
}

#[test]
fn test_connect_event_serialize() {
    let ref connector_event = event::ConnectorEvent {
        event_type: types::ConnectorEventType::Action(types::ActionType::INSERT),
        table_id: "tableId".to_string(),
        header_id: "headerId".to_string(),
        entries: vec![],
        old_values: vec![],
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


#[test]
fn test_serialize_table_event() {
    let ref graph_event = event::TableEvent::new(
        event::TableAction::FormulaSubmit {
            table_id: "tableId".to_string(),
            header_id: "headerId".to_string(),
            graph: default_formula_graph(),
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

#[test]
fn test_deserialize_table_event() {
    let data = include_bytes!("testfiles/test_table_event.json");
    let result = serde_json::from_slice::<event::TableEvent>(data);
    if result.is_err() {
        println!("{}", result.as_ref().unwrap_err())
    }
    assert!(result.is_ok());
    let table_event = result.unwrap();
    assert_eq!(table_event.get_key(), types::job_id("tableId", "headerId"));
    assert_eq!(
        table_event
            .event_time()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "2022-04-17T14:41:40.108Z"
    )
}

#[test]
fn test_deserialize_graph_event() {
    let data = include_bytes!("testfiles/test_graph_event.json");
    let result = serde_json::from_slice::<event::GraphEvent>(data);
    if result.is_err() {
        println!("{}", result.as_ref().unwrap_err())
    }
    assert!(result.is_ok());

    let graph_event = result.unwrap();
    match graph_event {
        GraphEvent::ExecutionGraphSubmit { ops, job_id } => {
            assert_eq!(job_id, types::job_id("tableId", "headerId"));
            assert_eq!(ops.job_id, types::job_id("tableId-1", "headerId-1"));
            assert_eq!(ops.meta, vec![
                types::AdjacentVec {
                    neighbors: vec![1, 2],
                    center: 0,
                },
                types::AdjacentVec {
                    neighbors: vec![3, 4],
                    center: 1,
                },
                types::AdjacentVec {
                    neighbors: vec![],
                    center: 2,
                },
                types::AdjacentVec {
                    neighbors: vec![],
                    center: 3,
                },
                types::AdjacentVec {
                    neighbors: vec![],
                    center: 4,
                },
            ]);
            assert_eq!(ops.nodes, types::NodeSet::from([
                (0.to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Reference {
                        table_id: "tableId-1".to_string(),
                        header_id: "headerId-1".to_string(),
                    },
                    id: 0,
                    upstream: vec![],
                }),
                (1.to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Sum,
                    id: 1,
                    upstream: vec![],
                }),
                (2.to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Sum,
                    id: 2,
                    upstream: vec![],
                }),
                (3.to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Sum,
                    id: 3,
                    upstream: vec![],
                }),
                (4.to_string(), types::Operator {
                    addr: "localhost".to_string(),
                    value: types::formula::FormulaOp::Sum,
                    id: 4,
                    upstream: vec![],
                })
            ]))
        }
        GraphEvent::DataSourceEventSubmit(_) => panic!("wrong type"),
        GraphEvent::TerminateGraph { .. } => panic!("wrong type"),
        GraphEvent::FormulaOpEventSubmit { .. } => panic!("wrong type"),
    }
}
use std::collections;
use dataflow::{runtime, types};

mod utils;

use utils::*;

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
        &default_nodeset(),
    );

    for id in 0..8 {
        assert!(addr_map.contains_key(&(id as u64)));
    }
}

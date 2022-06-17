use std::{cell, collections};
use std::ops::Deref;
use std::process::id;

use tokio::sync::mpsc;

use common::{hostname, local_ip};
use common::lists::{group_deque_btree_map, group_btree_map};

#[test]
fn test_local_ip() {
    let option = local_ip();
    assert!(option.is_some());
    println!("{}", option.unwrap())
}

#[test]
fn test_for_each() {
    let result = vec![1, 2, 3, 4];

    let mut index = 1;

    common::lists::for_each(&result, |int| {
        assert_eq!(int.clone(), index);
        index = index + 1;
    })
}

#[test]
fn test_map_self() {
    #[derive(Eq, PartialEq, Debug)]
    struct TestElem {
        pub id: u64,
        pub msg: String,
    }

    let ref list = vec![
        TestElem {
            id: 0,
            msg: "node-0".to_string(),
        },
        TestElem {
            id: 1,
            msg: "node-1".to_string(),
        },
        TestElem {
            id: 2,
            msg: "node-2".to_string(),
        },
    ];

    let map = common::lists::map_self(list, |elem| elem.id.clone());
    assert_eq!(
        map,
        collections::HashMap::from(
            [
                (0, &TestElem {
                    id: 0,
                    msg: "node-0".to_string(),
                }),
                (1, &TestElem {
                    id: 1,
                    msg: "node-1".to_string(),
                }),
                (2, &TestElem {
                    id: 2,
                    msg: "node-2".to_string(),
                })
            ]
        )
    )
}

#[test]
fn test_remove_if() {
    let ref mut vec = vec![1, 1, 2, 2, 3, 4, 5, 6];

    common::lists::remove_if(vec, |value| value.eq(&1));

    assert_eq!(vec, &mut vec![2, 2, 3, 4, 5, 6]);
}

#[test]
fn test_serde_env() {
    let origin = "{\"name\":\"${your.name}\", \"card\": \"${your.card}\"}";
    std::env::set_var("your.name", "jason");
    std::env::set_var("your.card", "111");
    let target = common::sysenv::serde_env::from_str(origin);
    let result = serde_json::from_str::<Name>(target.as_str());
    if result.is_err() {
        print!("{:?}", result.as_ref().unwrap_err())
    }
    assert!(result.is_ok());
    let name = result.unwrap();
    assert_eq!(&name.name, &"jason".to_string());
    assert_eq!(&name.card, &"111".to_string());
}

#[test]
fn test_regex() {
    let reg = regex::Regex::new("\\$\\{[^}]+\\}").unwrap();
    let option = reg.captures("\"name=\"${your.name}\"}");
    assert!(option.is_some());
    assert_eq!(option.as_ref().unwrap().len(), 1);
    assert_eq!(&option.as_ref().unwrap()[0], "${your.name}");
}

#[test]
fn test_env_var_get() {
    std::env::set_var("your.name", "jason");
    let result = std::env::var("your.name".to_string());
    assert!(result.is_ok());
    assert_eq!(result.as_ref().unwrap(), &"jason".to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Name {
    name: String,
    card: String,
}

#[test]
fn test_group_hashmap() {
    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Test {
        id: u64,
        name: String,
    }

    let ref list = vec![
        Test {
            id: 0,
            name: "name0".to_string(),
        },
        Test {
            id: 1,
            name: "name1".to_string(),
        },
        Test {
            id: 0,
            name: "name0-1".to_string(),
        },
        Test {
            id: 1,
            name: "name1-1".to_string(),
        },
        Test {
            id: 2,
            name: "name2".to_string(),
        },
    ];
    let map = common::lists::group_btree_map(list, |elem| elem.id.clone());
    assert!(map.contains_key(&0));
    assert!(map.contains_key(&1));
    assert!(map.contains_key(&2));
    assert_eq!(map.get(&0).unwrap(), &vec![
        &Test {
            id: 0,
            name: "name0".to_string(),
        },
        &Test {
            id: 0,
            name: "name0-1".to_string(),
        },
    ]);
    assert_eq!(map.get(&1).unwrap(), &vec![
        &Test {
            id: 1,
            name: "name1".to_string(),
        },
        &Test {
            id: 1,
            name: "name1-1".to_string(),
        },
    ]);
    assert_eq!(map.get(&2).unwrap(), &vec![
        &Test {
            id: 2,
            name: "name2".to_string(),
        },
    ])
}

#[test]
fn test_group_deque_hashmap() {
    use collections::{VecDeque, HashMap};

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct Test {
        id: u64,
        name: String,
    }

    let ref mut deque = VecDeque::from([
        Test {
            id: 0,
            name: "name0".to_string(),
        },
        Test {
            id: 1,
            name: "name1".to_string(),
        },
        Test {
            id: 0,
            name: "name0-1".to_string(),
        },
        Test {
            id: 1,
            name: "name1-1".to_string(),
        },
        Test {
            id: 2,
            name: "name2".to_string(),
        },
        Test {
            id: 2,
            name: "name2-1".to_string(),
        },
        Test {
            id: 3,
            name: "name3".to_string(),
        },
    ]);

    let map = group_deque_btree_map(deque, |elem| elem.id.clone());

    assert_eq!(map, HashMap::from([
        (0, vec![
            Test {
                id: 0,
                name: "name0".to_string(),
            },
            Test {
                id: 0,
                name: "name0-1".to_string(),
            },
        ]),
        (1, vec![
            Test {
                id: 1,
                name: "name1".to_string(),
            },
            Test {
                id: 1,
                name: "name1-1".to_string(),
            },
        ]),
        (2, vec![
            Test {
                id: 2,
                name: "name2".to_string(),
            },
            Test {
                id: 2,
                name: "name2-1".to_string(),
            },
        ]),
        (3, vec![
            Test {
                id: 3,
                name: "name3".to_string(),
            },
        ])
    ]));

    assert!(deque.is_empty())
}
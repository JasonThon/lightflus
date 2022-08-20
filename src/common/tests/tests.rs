use std::collections;

use common::collections::lang;

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

    let map = lang::map_self(list, |elem| elem.id.clone());
    assert_eq!(
        map,
        collections::HashMap::from([
            (
                0,
                &TestElem {
                    id: 0,
                    msg: "node-0".to_string(),
                }
            ),
            (
                1,
                &TestElem {
                    id: 1,
                    msg: "node-1".to_string(),
                }
            ),
            (
                2,
                &TestElem {
                    id: 2,
                    msg: "node-2".to_string(),
                }
            )
        ])
    )
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
    let map = lang::group(list, |elem| elem.id.clone());
    assert!(map.contains_key(&0));
    assert!(map.contains_key(&1));
    assert!(map.contains_key(&2));
    assert_eq!(
        map.get(&0).unwrap(),
        &vec![
            &Test {
                id: 0,
                name: "name0".to_string(),
            },
            &Test {
                id: 0,
                name: "name0-1".to_string(),
            },
        ]
    );
    assert_eq!(
        map.get(&1).unwrap(),
        &vec![
            &Test {
                id: 1,
                name: "name1".to_string(),
            },
            &Test {
                id: 1,
                name: "name1-1".to_string(),
            },
        ]
    );
    assert_eq!(
        map.get(&2).unwrap(),
        &vec![&Test {
            id: 2,
            name: "name2".to_string(),
        },]
    )
}

#[test]
fn test_group_deque_hashmap() {
    use collections::VecDeque;

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

    let map = lang::group_deque_as_btree_map(deque, |elem| elem.id.clone());

    assert_eq!(
        map,
        collections::BTreeMap::from([
            (
                0,
                vec![
                    Test {
                        id: 0,
                        name: "name0".to_string(),
                    },
                    Test {
                        id: 0,
                        name: "name0-1".to_string(),
                    },
                ]
            ),
            (
                1,
                vec![
                    Test {
                        id: 1,
                        name: "name1".to_string(),
                    },
                    Test {
                        id: 1,
                        name: "name1-1".to_string(),
                    },
                ]
            ),
            (
                2,
                vec![
                    Test {
                        id: 2,
                        name: "name2".to_string(),
                    },
                    Test {
                        id: 2,
                        name: "name2-1".to_string(),
                    },
                ]
            ),
            (
                3,
                vec![Test {
                    id: 3,
                    name: "name3".to_string(),
                },]
            )
        ])
    );

    assert!(deque.is_empty())
}

#[test]
fn test_any_match() {
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

    assert!(lang::any_match(list, |elem| elem.id == 0));
    assert!(lang::any_match(list, |elem| elem.id == 1));
    assert!(lang::any_match(list, |elem| elem.id == 2));
    assert!(lang::any_match(list, |elem| elem.name == "name2".to_string()));
    assert!(lang::any_match(list, |elem| elem.name == "name1".to_string()));
    assert!(lang::any_match(list, |elem| elem.name == "name1-1".to_string()));
    assert!(lang::any_match(list, |elem| elem.name == "name0".to_string()));
    assert!(lang::any_match(list, |elem| elem.name == "name0-1".to_string()));
}

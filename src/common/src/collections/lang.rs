use std::collections;
use std::collections::VecDeque;

pub fn map_self<N, T, F: FnMut(&N) -> T>(
    list: &Vec<N>,
    mut key_extractor: F,
) -> collections::HashMap<T, &N>
where
    T: std::hash::Hash + Eq,
{
    list.iter()
        .map(|elem| (key_extractor(elem), elem))
        .collect()
}

pub fn map<U, T, F: FnMut(&U) -> T>(list: &Vec<U>, mapper: F) -> Vec<T> {
    list.iter().map(mapper).collect()
}

pub fn group<N, T, F: FnMut(&N) -> T>(
    list: &Vec<N>,
    mut key_extractor: F,
) -> collections::BTreeMap<T, Vec<&N>>
where
    T: std::hash::Hash + Eq + Ord + Clone,
{
    list.iter()
        .map(|elem| collections::BTreeMap::from([(key_extractor(elem), vec![elem])]))
        .reduce(|mut accum, map| {
            map.iter().for_each(|elem| {
                let mut value_opts = accum.get_mut(&elem.0);
                value_opts
                    .iter_mut()
                    .for_each(|value| elem.1.iter().for_each(|e| value.push(*e)));

                if value_opts.is_none() {
                    accum.insert(elem.0.clone(), elem.1.clone());
                }
            });

            accum
        })
        .unwrap_or_else(|| Default::default())
}

/// group_deque will pop all elems from deque and group them as HashMap
/// ```
/// use std::collections;
/// use common::collections::lang;
///
/// let ref mut deque = collections::VecDeque::from([1,2,2,3,4,5]);
/// let map = lang::group_deque_as_btree_map(deque, |elem| elem.clone());
///
/// assert_eq!(map, collections::BTreeMap::from_iter([
///     (1,vec![1]), (2,vec![2,2]), (3,vec![3]), (4,vec![4]), (5, vec![5])
/// ]));
/// assert!(deque.is_empty())
/// ```
pub fn group_deque_as_btree_map<N, T, F: FnMut(&N) -> T>(
    deque: &mut VecDeque<N>,
    mut key_extractor: F,
) -> collections::BTreeMap<T, Vec<N>>
where
    T: std::hash::Hash + PartialEq + Ord,
{
    let mut result = collections::BTreeMap::new();

    while let Some(elem) = deque.pop_front() {
        let key = key_extractor(&elem);
        match result.get_mut(&key) {
            None => {
                result.insert(key, vec![elem]);
            }
            Some(values) => values.push(elem),
        }
    }

    result
}

pub fn any_match<T, F: FnMut(&T) -> bool>(elems: &Vec<T>, mut predicate: F) -> bool {
    elems.iter().filter(|e| predicate(*e)).next().is_some()
}

mod tests {

    #[test]
    fn test_map_self() {
        use std::collections;
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

        let map = super::map_self(list, |elem| elem.id.clone());
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
        let map = super::group(list, |elem| elem.id.clone());
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
        use std::collections;

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

        let map = super::group_deque_as_btree_map(deque, |elem| elem.id.clone());

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

        assert!(super::any_match(list, |elem| elem.id == 0));
        assert!(super::any_match(list, |elem| elem.id == 1));
        assert!(super::any_match(list, |elem| elem.id == 2));
        assert!(super::any_match(list, |elem| elem.name == "name2".to_string()));
        assert!(super::any_match(list, |elem| elem.name == "name1".to_string()));
        assert!(super::any_match(list, |elem| elem.name == "name1-1".to_string()));
        assert!(super::any_match(list, |elem| elem.name == "name0".to_string()));
        assert!(super::any_match(list, |elem| elem.name == "name0-1".to_string()));
    }
}

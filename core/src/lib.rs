use std::net;

pub mod http;
pub mod graph;
pub mod mongo;

pub trait KeyedValue<K, V> {
    fn key(&self) -> K;
    fn value(&self) -> V;
}

pub fn hostname() -> Option<String> {
    use std::process::Command;
    if cfg!(unix) || cfg!(windows) {
        let output = match Command::new("hostname").output() {
            Ok(o) => o,
            Err(_) => return None,
        };
        let mut s = String::from_utf8(output.stdout).unwrap();
        s.pop();  // pop '\n'
        Some(s)
    } else {
        None
    }
}

pub fn local_ip() -> Option<String> {
    let socket = match net::UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None
    };

    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };

    socket.local_addr()
        .ok()
        .map(|addr| addr.ip().to_string())
}

pub mod lists {
    use std::collections;

    pub fn contains<V, P: FnMut(&V) -> bool>(list: &Vec<V>, mut predicate: P) -> bool {
        for elem in list {
            if predicate(elem) {
                return true;
            }
        }

        false
    }

    pub fn for_each<V, F: FnMut(&V)>(list: &Vec<V>, mut f: F) {
        for elem in list {
            f(elem)
        }
    }

    pub fn for_each_mut<V, F: FnMut(&mut V)>(list: &mut Vec<V>, mut f: F) {
        for elem in list {
            f(elem)
        }
    }

    pub fn index_for_each<V, F: FnMut(usize, &V)>(list: &Vec<V>, mut f: F) {
        let mut index = 0 as usize;

        for elem in list {
            f(index.clone(), elem);
            index = index + 1;
        }
    }

    pub fn map<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut f: F) -> Vec<T> {
        let mut result = vec![];

        for elem in list {
            result.push(f(elem))
        }

        result
    }

    pub fn group_hashmap<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::HashMap<T, &N>
        where T: std::hash::Hash + std::cmp::Eq {
        let mut result = collections::HashMap::new();

        for elem in list {
            result.insert(key_extractor(elem), elem);
        }

        result
    }

    pub fn filter_map<N, T, F: FnMut(&N) -> T, Filter: FnMut(&N) -> bool>(
        list: &Vec<N>,
        mut filter: Filter,
        mut mapper: F) -> Vec<T> {
        let mut result = vec![];

        for elem in list {
            if filter(elem) {
                result.push(mapper(elem))
            }
        }

        result
    }

    pub fn remove_if<N, Filter: FnMut(&N) -> bool>(
        list: &mut Vec<N>,
        mut filter: Filter) {
        let mut index_vec = vec![];
        let mut offset = 0;

        for idx in 0..list.len() {
            if filter(&list[idx]) {
                index_vec.push(idx - offset);
                offset = offset + 1;
            }
        }

        for idx in &index_vec {
            list.remove(idx.clone() as usize);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{hostname, local_ip};
    use tokio::sync::mpsc;
    use std::{cell, collections};
    use std::ops::Deref;
    use std::process::id;

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

        super::lists::for_each(&result, |int| {
            assert_eq!(int.clone(), index);
            index = index + 1;
        })
    }

    #[test]
    fn test_channel_clone() {
        let (tx, mut rx) = mpsc::unbounded_channel::<usize>();

        let tx_1 = tx.clone();
        let tx_2 = tx.clone();

        tx_1.send(1);
        tx_2.send(2);
        tx.send(3);

        let result_1 = rx.try_recv();
        assert!(result_1.is_ok());
        let result_2 = rx.try_recv();
        assert!(result_2.is_ok());
        let result_3 = rx.try_recv();
        assert!(result_3.is_ok());

        assert_eq!(result_1.unwrap(), 1);
        assert_eq!(result_2.unwrap(), 2);
        assert_eq!(result_3.unwrap(), 3);
    }

    #[test]
    fn test_ref_mut() {
        let ref vec = vec![1, 2, 3, 4];

        let ref_cell = cell::RefCell::new(vec.clone());
        cell::RefMut::map(
            ref_cell.borrow_mut(),
            |v| {
                v.push(5);
                v
            },
        );

        println!("{:?}", std::time::SystemTime::now());

        assert_eq!(ref_cell.take(), vec![1, 2, 3, 4, 5])
    }

    #[test]
    fn test_unsafe_hashmap_get() {
        let ref_cell = cell::RefCell::new(collections::HashMap::new());

        cell::RefMut::map(
            ref_cell.borrow_mut(),
            |map| {
                map.insert(1, "James");
                map.insert(2, "Jason");
                map.insert(3, "Jone");
                map.insert(4, "Alice");

                map
            },
        );

        unsafe {
            let map = ref_cell.as_ptr().as_ref().unwrap();

            assert_eq!(map.get(&1), Some(&"James"));
            assert_eq!(map.get(&2), Some(&"Jason"));
            assert_eq!(map.get(&3), Some(&"Jone"));
            assert_eq!(map.get(&4), Some(&"Alice"));
        }
    }

    #[test]
    fn test_group() {
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

        let map = super::lists::group_hashmap(list, |elem| elem.id.clone());
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

        super::lists::remove_if(vec, |value| value.eq(&1));

        assert_eq!(vec, &mut vec![2, 2, 3, 4, 5, 6]);
    }
}
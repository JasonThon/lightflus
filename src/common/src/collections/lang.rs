use std::collections;
use std::collections::VecDeque;

pub fn any_match<V, P: FnMut(&V) -> bool>(list: &Vec<V>, predicate: P) -> bool {
    list.iter()
        .filter(predicate)
        .next()
        .is_some()
}

pub fn map_reduce<N, T, U: IntoIterator, F: FnMut(&N) -> U>(list: &Vec<N>, f: F) -> Vec<T> {
    list.iter()
        .flat_map(f)
        .collect()
}

pub fn map_self<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::HashMap<T, &N>
    where T: std::hash::Hash + std::cmp::Eq {
    let mut result = collections::HashMap::new();

    for elem in list {
        result.insert(key_extractor(elem), elem);
    }

    result
}

pub fn group_hashmap<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::HashMap<T, Vec<&N>>
    where T: std::hash::Hash + std::cmp::Eq {
    let mut result = collections::HashMap::new();

    list.iter().for_each(|elem| {
        let key = key_extractor(elem);

        if !result.contains_key(&key) {
            result.insert(key, vec![elem]);
        } else {
            result.get_mut(&key)
                .unwrap_or(&mut vec![])
                .push(elem);
        }
    });

    result
}

pub fn group_btree_map<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::BTreeMap<T, Vec<&N>>
    where T: std::hash::Hash + std::cmp::Eq + Ord {
    let mut result = collections::BTreeMap::new();
    for elem in list {
        let key = key_extractor(elem);

        if !result.contains_key(&key) {
            result.insert(key, vec![elem]);
        } else {
            result.get_mut(&key)
                .unwrap_or(&mut vec![])
                .push(elem);
        }
    }

    result
}

/// group_deque will pop all elems from deque and group them as HashMap
/// ```
/// use std::collections;
///
/// let ref mut deque = collections::VecDeque::from([1,2,2,3,4,5]);
/// let map = common::lang::group_deque_btree_map(deque, |elem| elem.clone());
///
/// assert_eq!(map, collections::BTreeMap::from_iter([
///     (1,vec![1]), (2,vec![2,2]), (3,vec![3]), (4,vec![4]), (5, vec![5])
/// ]));
/// assert!(deque.is_empty())
/// ```
pub fn group_deque_btree_map<N, T, F: FnMut(&N) -> T>(deque: &mut VecDeque<N>, mut key_extractor: F) -> collections::BTreeMap<T, Vec<N>>
    where T: std::hash::Hash + std::cmp::PartialEq + Ord {
    let mut result = collections::BTreeMap::new();

    while let Some(elem) = deque.pop_front() {
        let key = key_extractor(&elem);
        match result.get_mut(&key) {
            None => {
                result.insert(key, vec![elem]);
            }
            Some(values) => values.push(elem)
        }
    }

    result
}
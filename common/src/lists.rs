use std::collections;
use std::collections::VecDeque;

pub fn any_match<V, P: FnMut(&V) -> bool>(list: &Vec<V>, mut predicate: P) -> bool {
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

pub fn map_reduce<N, T, F: FnMut(&N) -> Vec<T>>(list: &Vec<N>, mut f: F) -> Vec<T> {
    let mut result = vec![];

    for elem in list {
        let ref mut values = f(elem);
        result.append(values)
    }

    result
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
/// let map = common::lists::group_deque_hashmap(deque, |elem| elem.clone());
///
/// assert_eq!(map, collections::HashMap::from_iter([
///     (1,vec![1]), (2,vec![2,2]), (3,vec![3]), (4,vec![4]), (5, vec![5])
/// ]));
/// assert!(deque.is_empty())
/// ```
pub fn group_deque_hashmap<N, T, F: FnMut(&N) -> T>(deque: &mut VecDeque<N>, mut key_extractor: F) -> collections::HashMap<T, Vec<N>>
    where T: std::hash::Hash + std::cmp::Eq {
    let mut result = collections::HashMap::new();

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

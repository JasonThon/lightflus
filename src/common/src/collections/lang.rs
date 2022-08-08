use std::collections;
use std::collections::VecDeque;

pub fn map_self<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::HashMap<T, &N>
    where T: std::hash::Hash + Eq {
    list.iter()
        .map(|elem| (key_extractor(elem), elem))
        .collect()
}

pub fn group<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut key_extractor: F) -> collections::BTreeMap<T, Vec<&N>>
    where T: std::hash::Hash + Eq + Ord + Clone {
    list.iter()
        .map(|elem| collections::BTreeMap::from([(key_extractor(elem), vec![elem])]))
        .reduce(|mut accum, map| {
            map.iter()
                .for_each(|elem| {
                    let mut value_opts = accum.get_mut(&elem.0);
                    value_opts.iter_mut()
                        .for_each(|value| elem.1
                            .iter()
                            .for_each(|e| value.push(*e)));

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
pub fn group_deque_as_btree_map<N, T, F: FnMut(&N) -> T>(deque: &mut VecDeque<N>, mut key_extractor: F) -> collections::BTreeMap<T, Vec<N>>
    where T: std::hash::Hash + PartialEq + Ord {
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
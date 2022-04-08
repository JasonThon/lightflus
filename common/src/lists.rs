use std::collections;

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

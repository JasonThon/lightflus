use std::hash::Hash;
use std::net;

pub mod http;
pub mod graph;

pub trait KeyedValue<K, V> {
    fn key(&self) -> K;
    fn value(&self) -> V;
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
    pub fn for_each<V, F: FnMut(&V)>(list: &Vec<V>, mut f: F) {
        for elem in list {
            f(elem)
        }
    }

    pub fn map<N, T, F: FnMut(&N) -> T>(list: &Vec<N>, mut f: F) -> Vec<T> {
        let mut result = vec![];

        for elem in list {
            result.push(f(elem))
        }

        result
    }
}

#[cfg(test)]
mod test {
    use crate::local_ip;

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
}
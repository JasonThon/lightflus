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
    use tokio::sync::mpsc;
    use std::cell;
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

        assert_eq!(ref_cell.take(), vec![1, 2, 3, 4, 5])
    }

    enum TestEnum {
        Vector {
            vec: Vec<u32>
        }
    }

    impl TestEnum {
        fn add_value(&mut self, value: u32) {
            match self {
                TestEnum::Vector {
                    vec
                } => vec.push(value)
            }
        }

        fn get(&self, idx: usize) -> Option<u32> {
            match self {
                TestEnum::Vector {
                    vec
                } => Option::Some(vec[idx])
            }
        }
    }
}
use std::marker;

pub trait StateManager<K, V>: Send + Sync {
    fn get_state(&self, key: &K) -> V;
}

pub struct RocksStateManager<K, V> {
    key: marker::PhantomData<K>,
    value: marker::PhantomData<V>,
}

unsafe impl<K, V> Send for RocksStateManager<K, V> {}

unsafe impl<K, V> Sync for RocksStateManager<K, V> {}


impl<K, V> StateManager<K, V> for RocksStateManager<K, V> {
    fn get_state(&self, key: &K) -> V {
        todo!()
    }
}

impl<K, V> RocksStateManager<K, V> {
    pub fn new() -> RocksStateManager<K, V> {
        RocksStateManager {
            key: Default::default(),
            value: Default::default(),
        }
    }
}
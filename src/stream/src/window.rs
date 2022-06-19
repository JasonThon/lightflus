use std::{collections, hash, marker, time};
use std::ops::Add;
use common::event;

#[derive(Clone)]
pub enum WindowType {
    Session {
        timeout: time::Duration
    },
    Fixed {
        size: time::Duration
    },
    Sliding {
        size: time::Duration,
        period: time::Duration,
    },
}

pub struct KeyedWindow<K, V> where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    pub values: Vec<V>,
    pub key: K,
    pub start: time::SystemTime,
    pub end: time::SystemTime,
    pub timestamp: time::SystemTime,
}

impl<K, V> Clone for KeyedWindow<K, V> where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            key: self.key.clone(),
            start: self.start,
            end: self.end,
            timestamp: self.timestamp,
        }
    }
}


impl<K, V> KeyedWindow<K, V> where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    pub(crate) fn merge(&mut self, other: &KeyedWindow<K, V>, window_type: &WindowType) {
        self.values.extend(other.values.to_vec());
        match window_type {
            WindowType::Session { .. } => {
                if self.end < other.end {
                    self.end = other.end
                }

                if self.start > other.start {
                    self.start = other.start
                }
            }
            _ => {}
        }
    }
}

unsafe impl<K, V> Send for KeyedWindow<K, V> where K: hash::Hash + Clone + Eq + Ord, V: Clone {}

unsafe impl<K, V> Sync for KeyedWindow<K, V> where K: hash::Hash + Clone + Ord, V: Clone {}

pub type KeyedWindowSet<K, V> = collections::HashMap<K, Vec<KeyedWindow<K, V>>>;

pub struct KeyedWindowAssigner<K, V, E: event::Event<K, V>> where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    window_type: WindowType,
    phantom_key: marker::PhantomData<K>,
    phantom_val: marker::PhantomData<V>,
    phantom_event: marker::PhantomData<E>,
}

impl<K, V, E: event::Event<K, V>> KeyedWindowAssigner<K, V, E> where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    pub fn assign(&self, datum: &E) -> Vec<KeyedWindow<K, V>> {
        let timestamp = time::SystemTime::from(datum.event_time());
        match &self.window_type {
            WindowType::Session { timeout } => {
                let end = timestamp.add(timeout.clone());
                vec![
                    KeyedWindow {
                        values: vec![datum.get_value()],
                        key: datum.get_key(),
                        start: timestamp.clone(),
                        end,
                        timestamp,
                    }
                ]
            }
            WindowType::Fixed { size } => {
                let end = timestamp.add(size.clone());
                vec![
                    KeyedWindow {
                        values: vec![datum.get_value()],
                        key: datum.get_key(),
                        start: timestamp.clone(),
                        end,
                        timestamp,
                    }
                ]
            }
            WindowType::Sliding { size, .. } => {
                let end = timestamp.add(size.clone());

                vec![KeyedWindow {
                    values: vec![datum.get_value()],
                    key: datum.get_key(),
                    start: timestamp.clone(),
                    end,
                    timestamp,
                }]
            }
        }
    }

    pub fn merge(&self, windows: &mut collections::VecDeque<KeyedWindow<K, V>>) {
        let ref mut group = collections::HashMap::<K, Vec<KeyedWindow<K, V>>>::new();
        while let Some(window) = windows.pop_front() {
            match group.get_mut(&window.key) {
                Some(wins) => {
                    match &self.window_type {
                        WindowType::Sliding { .. } => wins.push(window),
                        _ => {
                            let mut flag = false;

                            for win in wins.iter_mut() {
                                if win.end > window.timestamp && win.start <= window.timestamp {
                                    win.merge(&window, &self.window_type);
                                    flag = true;
                                    break;
                                }
                            }

                            if !flag {
                                wins.push(window.clone())
                            }
                        }
                    }
                }
                None => {
                    group.insert(window.key.clone(), vec![window]);
                }
            }
        }

        match &self.window_type {
            WindowType::Sliding { size, period } => {
                group.iter_mut()
                    .for_each(|(key, keyed_windows)| {
                        keyed_windows.sort_by_key(|win| win.timestamp);
                        let first_window = keyed_windows.first().unwrap();
                        let end_time = keyed_windows.last().unwrap().end;
                        let mut start = first_window.start;

                        while start < end_time {
                            let ref mut values = vec![];
                            for window in keyed_windows.into_iter() {
                                if window.timestamp >= start && window.timestamp < start + *size {
                                    for value in &window.values {
                                        values.push(value.clone())
                                    }
                                }
                            }
                            if !values.is_empty() {
                                windows.push_back(KeyedWindow {
                                    values: values.clone(),
                                    key: key.clone(),
                                    start,
                                    end: start + *size,
                                    timestamp: start,
                                });
                            }

                            start = start + *period;
                        }
                    })
            }
            _ => group
                .iter_mut()
                .for_each(|(_, values)| values
                    .iter()
                    .for_each(|win| windows.push_back(win.clone()))
                )
        }
    }
}

pub fn window_assigner<K, V, E: event::Event<K, V>>(window_type: &WindowType) -> KeyedWindowAssigner<K, V, E>
    where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    KeyedWindowAssigner {
        window_type: window_type.clone(),
        phantom_key: Default::default(),
        phantom_val: Default::default(),
        phantom_event: Default::default(),
    }
}

pub fn default_assigner<K, V, E: event::Event<K, V>>() -> KeyedWindowAssigner<K, V, E>
    where K: hash::Hash + Clone + Eq + Ord, V: Clone {
    KeyedWindowAssigner {
        window_type: WindowType::Fixed { size: time::Duration::from_millis(100) },
        phantom_key: Default::default(),
        phantom_val: Default::default(),
        phantom_event: Default::default(),
    }
}

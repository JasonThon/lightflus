use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign};
use std::time;
use common::event;
use stream::window::KeyedWindow;

struct MockEvent {
    key: String,
    value: String,
    event_time: chrono::DateTime<chrono::Utc>,
}

impl event::KeyedEvent<String, String> for MockEvent {
    fn event_time(&self) -> chrono::DateTime<chrono::Utc> {
        self.event_time.clone()
    }

    fn get_key(&self) -> String {
        self.key.clone()
    }

    fn get_value(&self) -> String {
        self.value.clone()
    }
}

#[test]
fn test_fixed_window_assign() {
    let ref fixed = stream::window::WindowType::Fixed { size: time::Duration::from_secs(2) };
    let ref start = time::SystemTime::now();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(fixed);
    let ref windows = assigner.assign(&MockEvent {
        key: "key".to_string(),
        value: "value".to_string(),
        event_time: chrono::DateTime::from(start.clone()),
    });

    assert_eq!(windows.len(), 1);

    let keyed_window = windows.get(0).unwrap();
    assert_eq!(keyed_window.key, "key");
    assert_eq!(keyed_window.values, vec!["value"]);
    assert_eq!(&keyed_window.start, start);
    assert_eq!(keyed_window.end, *start + time::Duration::from_secs(2));
    assert_eq!(keyed_window.timestamp, *start);
}

#[test]
fn test_sliding_window_assign() {
    let window_size = time::Duration::from_secs(5);
    let period = time::Duration::from_secs(2);
    let ref fixed = stream::window::WindowType::Sliding {
        size: window_size,
        period: period.clone(),
    };
    let ref start = time::SystemTime::now();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(fixed);
    let ref windows = assigner.assign(&MockEvent {
        key: "key".to_string(),
        value: "value".to_string(),
        event_time: chrono::DateTime::from(start.clone()),
    });

    assert_eq!(windows.len(), 1);
    let win = &windows[0];
    assert_eq!(win.key, "key");
    assert_eq!(win.values, vec!["value"]);
    assert_eq!(&win.start, start);
    assert_eq!(win.end, *start + window_size);
    assert_eq!(win.timestamp, *start);
}

#[test]
fn test_session_window_assign() {
    let timeout = time::Duration::from_secs(3);
    let ref session = stream::window::WindowType::Session { timeout: timeout.clone() };
    let ref start = time::SystemTime::now();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(session);
    let ref windows = assigner.assign(&MockEvent {
        key: "key".to_string(),
        value: "value".to_string(),
        event_time: chrono::DateTime::from(start.clone()),
    });

    assert_eq!(windows.len(), 1);
    let win = &windows[0];
    assert_eq!(win.key, "key");
    assert_eq!(win.values, vec!["value"]);
    assert_eq!(&win.start, start);
    assert_eq!(win.end, *start + timeout);
    assert_eq!(win.timestamp, *start);
}

#[test]
fn test_fixed_window_merge() {
    use std::collections::VecDeque;

    let ref fixed = stream::window::WindowType::Fixed { size: time::Duration::from_secs(2) };

    let ref mut start = time::SystemTime::now();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(fixed);
    let ref mut all_windows = VecDeque::new();

    for _ in 0..4 {
        all_windows.extend(
            assigner.assign(
                &MockEvent {
                    key: "key".to_string(),
                    value: "value".to_string(),
                    event_time: chrono::DateTime::from(start.clone()),
                }
            )
        );
        start.add_assign(time::Duration::from_secs(1));
    }

    assigner.merge(all_windows);

    for win in all_windows {
        assert_eq!(win.clone().values.len(), 2);
    }
}

#[test]
fn test_sliding_window_merge() {
    use std::collections::VecDeque;

    let ref sliding = stream::window::WindowType::Sliding {
        size: time::Duration::from_secs(6),
        period: time::Duration::from_secs(2),
    };

    let ref mut start = time::SystemTime::now();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(sliding);
    let ref mut all_windows = VecDeque::new();

    for _ in 0..12 {
        let windows = assigner.assign(
            &MockEvent {
                key: "key".to_string(),
                value: "value".to_string(),
                event_time: chrono::DateTime::from(start.clone()),
            }
        );
        for win in windows {
            all_windows.push_back(win);
        }

        start.add_assign(time::Duration::from_secs(2));
    }

    assigner.merge(all_windows);
    assert_eq!(all_windows.len(), 12);
    let mut idx = all_windows.len();
    for win in all_windows {
        if idx <= 2 {
            assert_eq!(win.values, vec!["value"; idx]);
        } else {
            assert_eq!(win.values, vec!["value"; 3]);
        }
        idx = idx - 1;
    }
}

#[test]
fn test_session_window_merge_out_timeout() {
    use std::collections::VecDeque;

    let ref session = stream::window::WindowType::Session {
        timeout: time::Duration::from_secs(3)
    };

    let ref mut start = time::SystemTime::now();
    let start_clone = start.clone();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(session);
    let ref mut all_windows = VecDeque::new();

    for _ in 0..5 {
        let wins = assigner.assign(&MockEvent {
            key: "key".to_string(),
            value: "value".to_string(),
            event_time: chrono::DateTime::from(start.clone()),
        });

        for win in wins {
            all_windows.push_back(win);
        }
        start.add_assign(time::Duration::from_secs(1));
    }

    start.add_assign(time::Duration::from_millis(2001));

    for _ in 0..5 {
        let wins = assigner.assign(&MockEvent {
            key: "key".to_string(),
            value: "value".to_string(),
            event_time: chrono::DateTime::from(start.clone()),
        });

        for win in wins {
            all_windows.push_back(win);
        }
        start.add_assign(time::Duration::from_secs(1));
    }

    assigner.merge(all_windows);
    assert_eq!(all_windows.len(), 2);
    let mut keyed_window = &all_windows[0];
    assert_eq!(keyed_window.values, vec!["value"; 5]);
    assert_eq!(keyed_window.key, "key");
    assert_eq!(keyed_window.start, start_clone);
    assert_eq!(keyed_window.end, start_clone + time::Duration::from_secs(7));
    keyed_window = &all_windows[1];

    assert_eq!(keyed_window.values, vec!["value"; 5]);
    assert_eq!(keyed_window.key, "key");
    assert_eq!(keyed_window.start, start_clone + time::Duration::from_secs(4) + time::Duration::from_millis(3001));
    assert_eq!(keyed_window.end, start_clone + time::Duration::from_secs(4) + time::Duration::from_millis(3001) + time::Duration::from_secs(7));

    assert_eq!(start.clone(), start_clone + time::Duration::from_secs(9) + time::Duration::from_millis(3001));
}

#[test]
fn test_session_window_merge_in_timeout() {
    use std::collections::VecDeque;

    let ref session = stream::window::WindowType::Session {
        timeout: time::Duration::from_secs(3)
    };

    let ref mut start = time::SystemTime::now();
    let start_clone = start.clone();
    let assigner = stream::window::window_assigner::<String, String, MockEvent>(session);
    let ref mut all_windows = VecDeque::new();

    for _ in 0..5 {
        let wins = assigner.assign(&MockEvent {
            key: "key".to_string(),
            value: "value".to_string(),
            event_time: chrono::DateTime::from(start.clone()),
        });

        for win in wins {
            all_windows.push_back(win);
        }
        start.add_assign(time::Duration::from_secs(1));
    }

    start.add_assign(time::Duration::from_millis(1000));

    for _ in 0..5 {
        let wins = assigner.assign(&MockEvent {
            key: "key".to_string(),
            value: "value".to_string(),
            event_time: chrono::DateTime::from(start.clone()),
        });

        for win in wins {
            all_windows.push_back(win);
        }
        start.add_assign(time::Duration::from_secs(1));
    }

    assigner.merge(all_windows);
    assert_eq!(all_windows.len(), 1);
    let ref mut keyed_window = all_windows[0];
    assert_eq!(keyed_window.values, vec!["value"; 10]);
    assert_eq!(keyed_window.key, "key");
    assert_eq!(keyed_window.start, start_clone);
    assert_eq!(keyed_window.end, start_clone + time::Duration::from_secs(10) + time::Duration::from_secs(3));
    assert_eq!(start.clone(), start_clone + time::Duration::from_secs(10) + time::Duration::from_millis(1000));
}
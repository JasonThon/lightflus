use std::{
    collections::{BTreeMap, VecDeque},
    task::{Context, Poll},
};

use chrono::Duration;
use common::collections::lang;

use proto::common::{keyed_data_event, KeyedDataEvent, Window};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use tokio::time::Instant;
use tonic::async_trait;

use crate::DETAULT_WATERMARK;

/// WindowAssigner implementations
pub enum WindowAssignerImpl {
    /// Fixed event-time-based keyed window assigner
    Fixed(FixedEventTimeKeyedWindowAssigner),
    /// Sliding event-time-based keyed window assigner
    Slide(SlideEventTimeKeyedWindowAssigner),
    /// Session event-time-based keyed window assigner
    Session(SessionKeyedWindowAssigner),
    /// a empty assigner
    Empty,
}

impl WindowAssignerImpl {
    pub fn new(window: &Window) -> Self {
        let trigger = window.get_trigger();
        match window.get_value() {
            Some(window) => match window {
                proto::common::window::Value::Fixed(fixed) => {
                    Self::Fixed(FixedEventTimeKeyedWindowAssigner::new(fixed, trigger))
                }
                proto::common::window::Value::Slide(slide) => {
                    Self::Slide(SlideEventTimeKeyedWindowAssigner::new(slide, trigger))
                }
                proto::common::window::Value::Session(session) => {
                    Self::Session(SessionKeyedWindowAssigner::new(session, trigger))
                }
            },
            None => Self::Empty,
        }
    }

    pub(crate) fn assign_windows(&self, keyed_event: KeyedDataEvent) -> Vec<KeyedWindow> {
        match self {
            Self::Fixed(assigner) => assigner.assign_windows(keyed_event),
            Self::Slide(assigner) => assigner.assign_windows(keyed_event),
            Self::Session(assigner) => assigner.assign_windows(keyed_event),
            Self::Empty => {
                let event_time = keyed_event.get_event_time();
                vec![KeyedWindow {
                    inner: keyed_event,
                    event_time,
                    window_start: 0,
                    window_end: 0,
                }]
            }
        }
    }

    /// GroupByKeyAndWindow operation
    /// In Dataflow Model, window merging is the part of this operation. This operation groups all windows by their key and window.
    /// The result of GroupByKeyAndWindow will be emited by Trigger. Trigger will support three refinement mode:
    /// 1. Discarding: this mode will discard all previous results;
    /// 2. Accumulating: this mode will store the window contents in a persistable state;
    /// 3. Accumulating & Retracting: this mode not only accumlate windows but also store the copy of emitted value of trigger;
    pub(crate) fn group_by_key_and_window(
        &self,
        keyed_windows: &mut VecDeque<KeyedWindow>,
    ) -> Vec<KeyedWindow> {
        match self {
            Self::Empty => keyed_windows.iter().map(|w| w.clone()).collect(),
            _ => {
                // MergeWindows
                let mut merged_windows = match self {
                    Self::Session(_) => KeyedWindow::merge_windows(keyed_windows),
                    _ => KeyedWindow::group_by_key(keyed_windows),
                };
                merged_windows
                    .par_iter_mut()
                    .map(|entry| {
                        let mut results = vec![];
                        // GroupAlsoByWindow
                        entry.1.sort_by(|a, b| b.window_start.cmp(&a.window_start));

                        while let Some(mut window) = entry.1.pop() {
                            if results.is_empty() {
                                results.push(window)
                            } else {
                                let mut filtered = results.iter_mut().filter(|w| {
                                    w.window_start == window.window_start
                                        && w.window_end == window.window_end
                                });
                                let mut opt = filtered.next();

                                if opt.is_none() {
                                    results.push(window)
                                } else {
                                    opt.iter_mut().for_each(|w| w.merge(&mut window));
                                }
                            }
                        }

                        results.iter_mut().for_each(|w| w.expand());

                        results
                    })
                    .reduce_with(|mut accum, mut current| {
                        // ExpandToElements
                        accum.append(&mut current);
                        accum
                    })
                    .unwrap_or_default()
            }
        }
    }

    pub(crate) fn poll_trigger(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        match self {
            Self::Fixed(fixed) => fixed.poll_trigger(cx),
            Self::Slide(slide) => slide.poll_trigger(cx),
            Self::Session(session) => session.poll_trigger(cx),
            Self::Empty => Poll::Ready(Instant::now()),
        }
    }
}

/// Base interface for Keyed window assigner
/// Before 1.0 version, only three event-time windows are supported:
/// 1. [`FixedEventTimeKeyedWindowAssigner`] support fixed event-time window
/// 2. [`SlideEventTimeKeyedWindowAssigner`] support sliding event-time window
/// 3. [`SessionKeyedWindowAssigner`] support session event-time window
#[async_trait]
pub trait KeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow>;

    async fn trigger(&mut self);

    fn poll_trigger(&mut self, cx: &mut Context<'_>) -> Poll<Instant>;
}

/// EventTime-based Fixed Window Assigner
/// Fixed window assigner will assign each event with fixed size and uninteracted windows
pub struct FixedEventTimeKeyedWindowAssigner {
    size: Duration,
    trigger: TriggerImpl,
}

impl FixedEventTimeKeyedWindowAssigner {
    fn new(
        fixed: &proto::common::window::FixedWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> FixedEventTimeKeyedWindowAssigner {
        let duration = fixed.get_size().to_duration();

        let trigger = trigger
            .and_then(|t| t.value.as_ref().map(|val| TriggerImpl::new(val)))
            .unwrap_or(TriggerImpl::default(&duration));

        FixedEventTimeKeyedWindowAssigner {
            size: duration,
            trigger,
        }
    }
}

#[async_trait]
impl KeyedWindowAssigner for FixedEventTimeKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let event_time = event.get_event_time();
        let window_start =
            KeyedWindow::get_window_start_with_offset(event_time, 0, self.size.num_milliseconds());

        let window_end = window_start + self.size.num_milliseconds();

        vec![KeyedWindow {
            inner: event,
            event_time: event_time.clone(),
            window_start,
            window_end,
        }]
    }

    async fn trigger(&mut self) {
        self.trigger.emit().await;
    }

    fn poll_trigger(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        self.trigger.poll_emit(cx)
    }
}

/// A unified structure for keyed window
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KeyedWindow {
    inner: KeyedDataEvent,
    pub event_time: i64,
    pub window_start: i64,
    pub window_end: i64,
}

impl KeyedWindow {
    pub(crate) fn get_window_start_with_offset(
        timestamp: i64,
        offset: i64,
        window_size: i64,
    ) -> i64 {
        let remainder = (timestamp - offset) % window_size;
        if remainder < 0 {
            timestamp - (remainder + window_size)
        } else {
            timestamp - remainder
        }
    }

    pub(crate) fn intersect(&self, window: &KeyedWindow) -> bool {
        self.window_end > window.window_start && self.window_start <= window.window_end
    }

    /// merge with another window
    /// After merged, the inner data of other window will be cleared
    pub(crate) fn merge(&mut self, other: &mut KeyedWindow) {
        self.inner.data.append(&mut other.inner.data);
        if self.window_start > other.window_start {
            self.window_start = other.window_start.clone()
        };
        if self.window_end < other.window_end {
            self.window_end = other.window_end.clone()
        }
        if self.inner.event_id < other.inner.event_id {
            self.inner.event_id = other.inner.event_id
        }
    }

    /// ExpandToElements operation
    pub(crate) fn expand(&mut self) {
        self.event_time = self.window_start;
    }

    /// merge all windows
    fn merge_windows(
        windows: &mut VecDeque<KeyedWindow>,
    ) -> BTreeMap<bytes::Bytes, Vec<KeyedWindow>> {
        // GroupByKey
        let mut group_by_key = Self::group_by_key(windows);
        group_by_key.par_iter_mut().for_each(|entry| {
            let mut results = vec![];
            entry.1.sort_by(|a, b| b.window_start.cmp(&a.window_start));

            while let Some(mut window) = entry.1.pop() {
                if results.is_empty() {
                    results.push(window);
                } else {
                    let mut filter = results.iter_mut().filter(|w| (*w).intersect(&window));
                    let mut next = filter.next();
                    if next.is_none() {
                        results.push(window);
                    } else {
                        next.iter_mut().for_each(|w| {
                            w.merge(&mut window);
                        })
                    }
                }
            }

            entry.1.append(&mut results);
        });

        group_by_key
    }

    fn group_by_key(
        windows: &mut VecDeque<KeyedWindow>,
    ) -> BTreeMap<bytes::Bytes, Vec<KeyedWindow>> {
        lang::group_deque_as_btree_map(windows, |item| {
            item.inner.window = None;
            item.inner.get_key().value.clone()
        })
    }

    pub fn to_event(mut self) -> KeyedDataEvent {
        self.inner.event_time = self.event_time;
        self.inner.window = Some(keyed_data_event::Window {
            start_time: self.window_start,
            end_time: self.window_end,
        });

        self.inner
    }
}

pub struct SlideEventTimeKeyedWindowAssigner {
    size: Duration,
    period: Duration,
    trigger: TriggerImpl,
}

impl SlideEventTimeKeyedWindowAssigner {
    fn new(
        slide: &proto::common::window::SlidingWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> SlideEventTimeKeyedWindowAssigner {
        let size = slide.get_size().to_duration();
        let period = slide.get_period().to_duration();
        let trigger = trigger
            .and_then(|t| t.value.as_ref().map(|value| TriggerImpl::new(value)))
            .unwrap_or_else(|| TriggerImpl::default(&size));

        SlideEventTimeKeyedWindowAssigner {
            size,
            period,
            trigger,
        }
    }
}

#[async_trait]
impl KeyedWindowAssigner for SlideEventTimeKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let mut results = vec![];
        let timestamp = event.get_event_time();
        let window_start =
            KeyedWindow::get_window_start_with_offset(timestamp, 0, self.size.num_milliseconds());
        let size = self.size.num_milliseconds();
        let event_time = event.get_event_time();
        let period = self.period.num_milliseconds();

        let mut start = window_start;
        while start > timestamp - size {
            results.push(KeyedWindow {
                inner: event.clone(),
                event_time,
                window_start: start,
                window_end: start + size,
            });
            start -= period;
        }

        results
    }

    async fn trigger(&mut self) {
        self.trigger.emit().await;
    }

    fn poll_trigger(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        self.trigger.poll_emit(cx)
    }
}

pub struct SessionKeyedWindowAssigner {
    timeout: Duration,
    trigger: TriggerImpl,
}

impl SessionKeyedWindowAssigner {
    fn new(
        session: &proto::common::window::SessionWindow,
        trigger: Option<&proto::common::Trigger>,
    ) -> SessionKeyedWindowAssigner {
        let timeout = session.get_timeout().to_duration();
        SessionKeyedWindowAssigner {
            trigger: trigger
                .and_then(|t| t.value.as_ref().map(|value| TriggerImpl::new(value)))
                .unwrap_or_else(|| TriggerImpl::default(&timeout)),
            timeout,
        }
    }
}

#[async_trait]
impl KeyedWindowAssigner for SessionKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let event_time = event.get_event_time();
        let window_start = event_time.clone();
        let window_end = event_time + self.timeout.num_milliseconds();
        vec![KeyedWindow {
            inner: event,
            event_time,
            window_start,
            window_end,
        }]
    }

    async fn trigger(&mut self) {
        self.trigger.emit().await;
    }

    fn poll_trigger(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        self.trigger.poll_emit(cx)
    }
}

pub enum TriggerImpl {
    Watermark { timer: tokio::time::Interval },
}

impl TriggerImpl {
    fn new(val: &proto::common::trigger::Value) -> Self {
        match val {
            proto::common::trigger::Value::Watermark(watermark) => {
                let trigger_time = watermark.get_trigger_time().to_duration();

                Self::new_watermark(&trigger_time)
            }
        }
    }

    fn default(duration: &Duration) -> Self {
        Self::new_watermark(&duration)
    }

    fn new_watermark(duration: &Duration) -> Self {
        let trigger_time = duration.to_std().unwrap_or(DETAULT_WATERMARK);

        Self::Watermark {
            timer: tokio::time::interval(trigger_time),
        }
    }

    async fn emit(&mut self) {
        match self {
            Self::Watermark { timer, .. } => {
                timer.reset();
                let instant = timer.tick().await.into_std();
                tracing::info!("watermark trigger emit at {:?}", &instant);
            }
        }
    }

    fn poll_emit(&mut self, cx: &mut Context) -> Poll<Instant> {
        match self {
            Self::Watermark { timer } => {
                timer.reset();
                timer.poll_tick(cx)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        time::{Duration, Instant},
    };

    use common::utils::times::timestamp;
    use prost_types::Timestamp;
    use proto::common::{
        window::{FixedWindow, SessionWindow, SlidingWindow},
        Entry, KeyedDataEvent, Time,
    };

    use crate::window::{KeyedWindow, WindowAssignerImpl};

    use super::TriggerImpl;

    #[tokio::test]
    async fn test_fixed_window_assigner() {
        let fixed = FixedWindow {
            size: Some(Time {
                millis: 300,
                seconds: 0,
                minutes: 0,
                hours: 0,
            }),
        };

        let assigner =
            WindowAssignerImpl::Fixed(super::FixedEventTimeKeyedWindowAssigner::new(&fixed, None));

        fn test_assign_windows(assigner: &WindowAssignerImpl) {
            let now = chrono::Utc::now();
            let event = KeyedDataEvent {
                job_id: None,
                key: None,
                to_operator_id: 1,
                data: vec![],
                event_time: now.timestamp_millis(),
                from_operator_id: 0,
                window: None,
                event_id: 1,
            };

            let windows = assigner.assign_windows(event.clone());
            let window_start =
                KeyedWindow::get_window_start_with_offset(now.timestamp_millis(), 0, 300);

            assert_eq!(
                windows,
                vec![KeyedWindow {
                    inner: event,
                    event_time: timestamp(&now),
                    window_start: window_start,
                    window_end: window_start + 300
                }]
            );
        }

        test_assign_windows(&assigner);
    }

    #[tokio::test]
    async fn test_sliding_window_assigner() {
        let sliding = SlidingWindow {
            size: Some(Time {
                millis: 300,
                seconds: 0,
                minutes: 0,
                hours: 0,
            }),
            period: Some(Time {
                millis: 100,
                seconds: 0,
                minutes: 0,
                hours: 0,
            }),
        };

        let assigner = WindowAssignerImpl::Slide(super::SlideEventTimeKeyedWindowAssigner::new(
            &sliding, None,
        ));

        fn test_assign_window(assigner: &WindowAssignerImpl, window_size: i64, period: i64) {
            let now = chrono::Utc::now();
            let event = KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![3]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![7]),
                }],
                event_time: timestamp(&now),
                from_operator_id: 0,
                window: None,
                event_id: 1,
            };
            let windows = assigner.assign_windows(event);
            let window_start =
                KeyedWindow::get_window_start_with_offset(now.timestamp_millis(), 0, window_size);
            let num = (window_start - now.timestamp_millis() + window_size) / period + 1;

            assert!(windows.len() > 0);
            assert_eq!(windows.len(), num as usize);
            (0..num).into_iter().for_each(|index| {
                let start = window_start - period * index;
                assert_eq!(
                    &windows[index as usize],
                    &KeyedWindow {
                        inner: KeyedDataEvent {
                            job_id: None,
                            key: Some(Entry {
                                data_type: 1,
                                value: bytes::Bytes::from(vec![3])
                            }),
                            to_operator_id: 1,
                            data: vec![Entry {
                                data_type: 1,
                                value: bytes::Bytes::from(vec![7])
                            }],
                            event_time: timestamp(&now),
                            from_operator_id: 0,
                            window: None,
                            event_id: 1
                        },
                        event_time: timestamp(&now),
                        window_start: start,
                        window_end: start + window_size
                    }
                )
            });
        }

        test_assign_window(
            &assigner,
            sliding.get_size().to_duration().num_milliseconds(),
            sliding.get_period().to_duration().num_milliseconds(),
        );
    }

    #[tokio::test]
    async fn test_session_window_assigner() {
        let session = SessionWindow {
            timeout: Some(Time {
                millis: 0,
                seconds: 3,
                minutes: 0,
                hours: 0,
            }),
        };

        let assigner =
            WindowAssignerImpl::Session(super::SessionKeyedWindowAssigner::new(&session, None));
        let now = chrono::Utc::now();
        let event = KeyedDataEvent {
            job_id: None,
            key: Some(Entry {
                data_type: 1,
                value: bytes::Bytes::from(vec![3]),
            }),
            to_operator_id: 1,
            data: vec![Entry {
                data_type: 1,
                value: bytes::Bytes::from(vec![7]),
            }],
            event_time: timestamp(&now),
            from_operator_id: 0,
            window: None,
            event_id: 1,
        };
        let windows = assigner.assign_windows(event.clone());
        assert_eq!(
            windows,
            vec![KeyedWindow {
                inner: event,
                event_time: timestamp(&now),
                window_start: timestamp(&now),
                window_end: timestamp(&now)
                    + session.get_timeout().to_duration().num_milliseconds()
            }]
        )
    }

    #[tokio::test]
    async fn test_empty_window_assigner() {
        let assigner = WindowAssignerImpl::Empty;
        let now = chrono::Utc::now();
        let event = KeyedDataEvent {
            job_id: None,
            key: Some(Entry {
                data_type: 1,
                value: bytes::Bytes::from(vec![3]),
            }),
            to_operator_id: 1,
            data: vec![Entry {
                data_type: 1,
                value: bytes::Bytes::from(vec![7]),
            }],
            event_time: timestamp(&now),
            from_operator_id: 0,
            window: None,
            event_id: 1,
        };
        let windows = assigner.assign_windows(event.clone());
        let event_time = event.get_event_time();
        assert_eq!(
            windows,
            vec![KeyedWindow {
                inner: event,
                event_time,
                window_start: 0,
                window_end: 0,
            }]
        )
    }

    #[tokio::test]
    async fn test_group_by_key_and_window() {
        let fixed = FixedWindow {
            size: Some(Time {
                millis: 300,
                seconds: 0,
                minutes: 0,
                hours: 0,
            }),
        };

        let assigner =
            WindowAssignerImpl::Fixed(super::FixedEventTimeKeyedWindowAssigner::new(&fixed, None));
        let now = chrono::Utc::now();

        // unordered event input
        let events = vec![
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![1]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![2]),
                }],
                event_time: timestamp(&now),
                from_operator_id: 0,
                window: None,
                event_id: 1,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![1]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![3]),
                }],
                event_time: timestamp(&now) + 35,
                from_operator_id: 0,
                window: None,
                event_id: 2,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![1]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![4]),
                }],
                event_time: timestamp(&now) + 10,
                from_operator_id: 0,
                window: None,
                event_id: 3,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![2]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![5]),
                }],
                event_time: timestamp(&now) + 20,
                from_operator_id: 0,
                window: None,
                event_id: 4,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![2]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![6]),
                }],
                event_time: 350 + timestamp(&now),
                from_operator_id: 0,
                window: None,
                event_id: 5,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![3]),
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: bytes::Bytes::from(vec![7]),
                }],
                event_time: 250 + timestamp(&now),
                from_operator_id: 0,
                window: None,
                event_id: 6,
            },
        ];

        let windows = events
            .iter()
            .map(|event| assigner.assign_windows(event.clone()))
            .reduce(|mut accum, mut current| {
                accum.append(&mut current);
                accum
            });

        assert!(windows.is_some());
        let windows = windows.unwrap();

        let windows = assigner.group_by_key_and_window(&mut VecDeque::from(windows));
        let window_size = fixed.get_size().to_duration().num_milliseconds();

        {
            let mut key_1_filter = windows.iter().filter(|w| {
                w.inner.key
                    == Some(Entry {
                        data_type: 1,
                        value: bytes::Bytes::from(vec![1]),
                    })
            });

            let next = key_1_filter.next();
            let window_start =
                KeyedWindow::get_window_start_with_offset(now.timestamp_millis(), 0, window_size);
            assert_eq!(
                next,
                Some(&KeyedWindow {
                    inner: KeyedDataEvent {
                        job_id: None,
                        key: Some(Entry {
                            data_type: 1,
                            value: bytes::Bytes::from(vec![1])
                        }),
                        to_operator_id: 1,
                        data: vec![
                            Entry {
                                data_type: 1,
                                value: bytes::Bytes::from(vec![4])
                            },
                            Entry {
                                data_type: 1,
                                value: bytes::Bytes::from(vec![3])
                            },
                            Entry {
                                data_type: 1,
                                value: bytes::Bytes::from(vec![2])
                            }
                        ],
                        event_time: 10 + timestamp(&now),
                        from_operator_id: 0,
                        window: None,
                        event_id: 3,
                    },
                    event_time: window_start,
                    window_start: window_start,
                    window_end: window_start + window_size
                })
            )
        }

        {
            let mut key_3_filter = windows.iter().filter(|w| {
                w.inner.key
                    == Some(Entry {
                        data_type: 1,
                        value: bytes::Bytes::from(vec![3]),
                    })
            });

            let next = key_3_filter.next();
            let window_start = now
                .checked_add_signed(chrono::Duration::milliseconds(250))
                .map(|event_time| {
                    KeyedWindow::get_window_start_with_offset(
                        event_time.timestamp_millis(),
                        0,
                        window_size,
                    )
                });
            assert_eq!(
                next,
                Some(&KeyedWindow {
                    inner: KeyedDataEvent {
                        job_id: None,
                        key: Some(Entry {
                            data_type: 1,
                            value: bytes::Bytes::from(vec![3])
                        }),
                        to_operator_id: 1,
                        data: vec![Entry {
                            data_type: 1,
                            value: bytes::Bytes::from(vec![7])
                        }],
                        event_time: 250 + timestamp(&now),
                        from_operator_id: 0,
                        window: None,
                        event_id: 6
                    },
                    event_time: window_start.unwrap_or_default(),
                    window_start: window_start.unwrap_or_default(),
                    window_end: window_start.unwrap_or_default() + 300
                })
            )
        }
    }

    #[tokio::test]
    async fn test_watermark_trigger() {
        let val = proto::common::trigger::Value::Watermark(proto::common::trigger::Watermark {
            trigger_time: Some(Time {
                millis: 200,
                seconds: 1,
                minutes: 0,
                hours: 0,
            }),
        });
        let mut trigger = TriggerImpl::new(&val);
        let start = Instant::now();
        trigger.emit().await;
        let end_instant = Instant::now();
        let elapsed_time = end_instant.duration_since(start);
        let result = elapsed_time
            >= Duration::from_secs(1)
                .checked_add(Duration::from_millis(200))
                .unwrap_or_default();
        assert!(result);
    }
}

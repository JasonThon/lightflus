use std::collections::{BTreeMap, VecDeque};

use chrono::Duration;
use common::{collections::lang, utils::times::from_millis_to_utc_chrono};
use prost_types::Timestamp;
use proto::common::{keyed_data_event, KeyedDataEvent, OperatorInfo};
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

use crate::DETAULT_WATERMARK;

/// WindowAssigner implementations
pub enum WindowAssignerImpl {
    Fixed(FixedEventTimeKeyedWindowAssigner),
    Slide(SlideEventTimeKeyedWindowAssigner),
    Session(SessionKeyedWindowAssigner),
    Empty,
}

impl WindowAssignerImpl {
    pub fn new(operator: &OperatorInfo) -> Self {
        let window = operator.get_window();
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
                    window_start: None,
                    window_end: None,
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

    pub(crate) async fn trigger(&mut self) {
        match self {
            WindowAssignerImpl::Fixed(fixed) => fixed.trigger().await,
            WindowAssignerImpl::Slide(slide) => slide.trigger().await,
            WindowAssignerImpl::Session(session) => session.trigger().await,
            WindowAssignerImpl::Empty => {}
        }
    }
}

/// Base interface for Keyed window assigner
/// Before 1.0 version, only three event-time windows are supported:
/// 1. [`FixedEventTimeKeyedWindowAssigner`] support fixed event-time window
/// 2. [`SlideEventTimeKeyedWindowAssigner`] support sliding event-time window
/// 3. [`SessionKeyedWindowAssigner`] support session event-time window
#[async_trait::async_trait]
pub trait KeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow>;

    async fn trigger(&mut self);
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

#[async_trait::async_trait]
impl KeyedWindowAssigner for FixedEventTimeKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let event_time = event.get_event_time();
        let window_start = event_time
            .as_ref()
            .map(|datetime| {
                KeyedWindow::get_window_start_with_offset(
                    datetime.timestamp_millis(),
                    0,
                    self.size.num_milliseconds(),
                )
            })
            .and_then(|start| from_millis_to_utc_chrono(start));

        let window_end = window_start
            .as_ref()
            .and_then(|start_time| start_time.checked_add_signed(self.size));
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
}

/// A unified structure for keyed window
#[derive(Clone, PartialEq, Debug, Default)]
pub struct KeyedWindow {
    inner: KeyedDataEvent,
    pub event_time: Option<chrono::DateTime<chrono::Utc>>,
    pub window_start: Option<chrono::DateTime<chrono::Utc>>,
    pub window_end: Option<chrono::DateTime<chrono::Utc>>,
}

impl KeyedWindow {
    pub(crate) fn as_event(&mut self) -> &KeyedDataEvent {
        self.inner.event_time = self.event_time.map(|time| Timestamp {
            seconds: time.timestamp(),
            nanos: time.timestamp_subsec_nanos() as i32,
        });

        self.inner.window = self
            .window_end
            .iter()
            .zip(self.window_start.iter())
            .map(|pair| keyed_data_event::Window {
                start_time: Some(Timestamp {
                    seconds: pair.1.timestamp(),
                    nanos: pair.1.timestamp_subsec_nanos() as i32,
                }),
                end_time: Some(Timestamp {
                    seconds: pair.0.timestamp(),
                    nanos: pair.0.timestamp_subsec_nanos() as i32,
                }),
            })
            .next();

        &self.inner
    }

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
    }

    /// ExpandToElements operation
    pub(crate) fn expand(&mut self) {
        self.event_time = self.window_start.clone();
    }

    /// merge all windows
    fn merge_windows(windows: &mut VecDeque<KeyedWindow>) -> BTreeMap<Vec<u8>, Vec<KeyedWindow>> {
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

    fn group_by_key(windows: &mut VecDeque<KeyedWindow>) -> BTreeMap<Vec<u8>, Vec<KeyedWindow>> {
        lang::group_deque_as_btree_map(windows, |item| {
            item.inner.window = None;
            item.inner.get_key().value.clone()
        })
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

#[async_trait::async_trait]
impl KeyedWindowAssigner for SlideEventTimeKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let mut results = vec![];
        let timestamp = event
            .get_event_time()
            .map(|event_time| event_time.timestamp_millis());
        let window_start = timestamp.map(|ts| {
            KeyedWindow::get_window_start_with_offset(ts, 0, self.size.num_milliseconds())
        });
        let size = self.size.num_milliseconds();
        let event_time = event.get_event_time();
        let period = self.period.num_milliseconds();

        timestamp
            .iter()
            .zip(window_start.iter())
            .for_each(|(timestamp, window_start)| {
                let mut start = *window_start;
                while start > timestamp - size {
                    results.push(KeyedWindow {
                        inner: event.clone(),
                        event_time: event_time.clone(),
                        window_start: from_millis_to_utc_chrono(start),
                        window_end: from_millis_to_utc_chrono(start + size),
                    });
                    start -= period;
                }
            });

        results
    }

    async fn trigger(&mut self) {
        self.trigger.emit().await;
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

#[async_trait::async_trait]
impl KeyedWindowAssigner for SessionKeyedWindowAssigner {
    fn assign_windows(&self, event: KeyedDataEvent) -> Vec<KeyedWindow> {
        let event_time = event.get_event_time();
        let window_start = event_time.clone();
        let window_end = event_time
            .as_ref()
            .and_then(|t| t.checked_add_signed(self.timeout));
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
            TriggerImpl::Watermark { timer, .. } => {
                timer.reset();
                let instant = timer.tick().await.into_std();
                tracing::info!("watermark trigger emit at {:?}", &instant);
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

    use common::utils::times::from_millis_to_utc_chrono;
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
            let event_time = Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            };
            let event = KeyedDataEvent {
                job_id: None,
                key: None,
                to_operator_id: 1,
                data: vec![],
                event_time: Some(event_time),
                process_time: None,
                from_operator_id: 0,
                window: None,
            };

            let windows = assigner.assign_windows(event.clone());
            let window_start =
                KeyedWindow::get_window_start_with_offset(now.timestamp_millis(), 0, 300);

            assert_eq!(
                windows,
                vec![KeyedWindow {
                    inner: event,
                    event_time: Some(now.clone()),
                    window_start: from_millis_to_utc_chrono(window_start),
                    window_end: from_millis_to_utc_chrono(window_start + 300)
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
                    value: vec![3],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![7],
                }],
                event_time: Some(Timestamp {
                    seconds: now.timestamp(),
                    nanos: now.timestamp_subsec_nanos() as i32,
                }),
                process_time: None,
                from_operator_id: 0,
                window: None,
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
                                value: vec![3]
                            }),
                            to_operator_id: 1,
                            data: vec![Entry {
                                data_type: 1,
                                value: vec![7]
                            }],
                            event_time: Some(Timestamp {
                                seconds: now.timestamp(),
                                nanos: now.timestamp_subsec_nanos() as i32,
                            }),
                            process_time: None,
                            from_operator_id: 0,
                            window: None
                        },
                        event_time: Some(now.clone()),
                        window_start: from_millis_to_utc_chrono(start),
                        window_end: from_millis_to_utc_chrono(start + window_size)
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
                value: vec![3],
            }),
            to_operator_id: 1,
            data: vec![Entry {
                data_type: 1,
                value: vec![7],
            }],
            event_time: Some(Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            }),
            process_time: None,
            from_operator_id: 0,
            window: None,
        };
        let windows = assigner.assign_windows(event.clone());
        assert_eq!(
            windows,
            vec![KeyedWindow {
                inner: event,
                event_time: Some(now.clone()),
                window_start: Some(now.clone()),
                window_end: now.checked_add_signed(session.get_timeout().to_duration())
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
                value: vec![3],
            }),
            to_operator_id: 1,
            data: vec![Entry {
                data_type: 1,
                value: vec![7],
            }],
            event_time: Some(Timestamp {
                seconds: now.timestamp(),
                nanos: now.timestamp_subsec_nanos() as i32,
            }),
            process_time: None,
            from_operator_id: 0,
            window: None,
        };
        let windows = assigner.assign_windows(event.clone());
        let event_time = event.get_event_time();
        assert_eq!(
            windows,
            vec![KeyedWindow {
                inner: event,
                event_time,
                window_start: None,
                window_end: None,
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
                    value: vec![1],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![2],
                }],
                event_time: Some(Timestamp {
                    seconds: now.timestamp(),
                    nanos: now.timestamp_subsec_nanos() as i32,
                }),
                process_time: None,
                from_operator_id: 0,
                window: None,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: vec![1],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![3],
                }],
                event_time: now
                    .checked_add_signed(chrono::Duration::milliseconds(35))
                    .map(|t| Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: 0,
                window: None,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: vec![1],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![4],
                }],
                event_time: now
                    .checked_add_signed(chrono::Duration::milliseconds(10))
                    .map(|t| Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: 0,
                window: None,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: vec![2],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![5],
                }],
                event_time: now
                    .checked_add_signed(chrono::Duration::milliseconds(20))
                    .map(|t| Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: 0,
                window: None,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: vec![2],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![6],
                }],
                event_time: now
                    .checked_add_signed(chrono::Duration::milliseconds(350))
                    .map(|t| Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: 0,
                window: None,
            },
            KeyedDataEvent {
                job_id: None,
                key: Some(Entry {
                    data_type: 1,
                    value: vec![3],
                }),
                to_operator_id: 1,
                data: vec![Entry {
                    data_type: 1,
                    value: vec![7],
                }],
                event_time: now
                    .checked_add_signed(chrono::Duration::milliseconds(250))
                    .map(|t| Timestamp {
                        seconds: t.timestamp(),
                        nanos: t.timestamp_subsec_nanos() as i32,
                    }),
                process_time: None,
                from_operator_id: 0,
                window: None,
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
                        value: vec![1],
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
                            value: vec![1]
                        }),
                        to_operator_id: 1,
                        data: vec![
                            Entry {
                                data_type: 1,
                                value: vec![4]
                            },
                            Entry {
                                data_type: 1,
                                value: vec![3]
                            },
                            Entry {
                                data_type: 1,
                                value: vec![2]
                            }
                        ],
                        event_time: Some(Timestamp {
                            seconds: now.timestamp(),
                            nanos: (now.timestamp_subsec_nanos() + 10 * common::NANOS_PER_MILLI)
                                as i32
                        }),
                        process_time: None,
                        from_operator_id: 0,
                        window: None
                    },
                    event_time: from_millis_to_utc_chrono(window_start),
                    window_start: from_millis_to_utc_chrono(window_start),
                    window_end: from_millis_to_utc_chrono(window_start + window_size)
                })
            )
        }

        {
            let mut key_3_filter = windows.iter().filter(|w| {
                w.inner.key
                    == Some(Entry {
                        data_type: 1,
                        value: vec![3],
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
                            value: vec![3]
                        }),
                        to_operator_id: 1,
                        data: vec![Entry {
                            data_type: 1,
                            value: vec![7]
                        }],
                        event_time: now
                            .checked_add_signed(chrono::Duration::milliseconds(250))
                            .map(|t| Timestamp {
                                seconds: t.timestamp(),
                                nanos: t.timestamp_subsec_nanos() as i32,
                            }),
                        process_time: None,
                        from_operator_id: 0,
                        window: None
                    },
                    event_time: window_start
                        .and_then(|timestamp| from_millis_to_utc_chrono(timestamp)),
                    window_start: window_start
                        .and_then(|timestamp| from_millis_to_utc_chrono(timestamp)),
                    window_end: window_start
                        .and_then(|timestamp| from_millis_to_utc_chrono(timestamp + 300))
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

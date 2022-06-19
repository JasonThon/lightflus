use std::ops::Add;
use stream::pipeline::Pipeline;
use common::types::job_id;
use std::time;
use common::event::FormulaOpEvent;

#[test]
fn test_apply_reference_insert() {
    use stream::pipeline::FormulaOpEventPipeline;
    use stream::window::KeyedWindow;
    use common::types::{formula::FormulaOp, ActionValue, ActionType, TypedValue};

    let p = FormulaOpEventPipeline::new(
        FormulaOp::Reference { table_id: "tableId".to_string(), header_id: "headerId".to_string() },
        job_id("tableId-1", "headerId-1"),
        0,
        vec![],
    );
    let ref context = p.create_context();
    let start = time::SystemTime::now();
    let ref win = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(400)),
        timestamp: start.add(time::Duration::from_millis(200)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 0,
            }
        ],
    };
    let result = p.apply(win, context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![
        FormulaOpEvent {
            row_idx: 0,
            job_id: job_id("tableId-1", "headerId-1"),
            data: TypedValue::Int(10).get_data(),
            old_data: vec![],
            from: 0,
            action: ActionType::INSERT,
            event_time: start.add(time::Duration::from_millis(200)),
        }
    ]);
    assert!(context.get_state(&0).is_none())
}

#[test]
fn test_add_without_values_no_delayed_event_arrival() {
    use stream::pipeline::FormulaOpEventPipeline;
    use stream::window::KeyedWindow;
    use common::types::{formula::FormulaOp, ActionValue, ActionType, TypedValue, FormulaState, ValueState};

    let p = FormulaOpEventPipeline::new(
        FormulaOp::Add { values: vec![] },
        job_id("tableId-1", "headerId-1"),
        3,
        vec![1, 2],
    );

    let ref context = p.create_context();
    let start = time::SystemTime::now();
    let ref insert_wind = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(400)),
        timestamp: start.add(time::Duration::from_millis(200)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 1,
            },
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 2,
            },
        ],
    };
    let result = p.apply(insert_wind, context);
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events, vec![
        FormulaOpEvent {
            row_idx: 0,
            job_id: job_id("tableId-1", "headerId-1"),
            data: TypedValue::Int(20).get_data(),
            old_data: vec![],
            from: 3,
            action: ActionType::INSERT,
            event_time: events[0].event_time,
        }
    ]);
    let state_opt = context.get_state(&0);
    assert!(state_opt.is_some());
    let mut states = state_opt.unwrap();
    states.node_states.sort_by_key(|v| v.node_idx);
    assert_eq!(states, FormulaState {
        value: TypedValue::Int(20).get_data(),
        node_states: vec![
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 1,
            },
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 2,
            },
        ],
    });
}

#[test]
fn test_add_without_values_delayed_insert_event_arrival() {
    use stream::pipeline::FormulaOpEventPipeline;
    use stream::window::KeyedWindow;
    use common::types::{formula::FormulaOp, ActionValue, ActionType, TypedValue, FormulaState, ValueState};

    let p = FormulaOpEventPipeline::new(
        FormulaOp::Add { values: vec![] },
        job_id("tableId-1", "headerId-1"),
        3,
        vec![1, 2],
    );

    let ref context = p.create_context();
    let start = time::SystemTime::now();
    let ref insert_wind = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(400)),
        timestamp: start.add(time::Duration::from_millis(200)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 1,
            },
        ],
    };
    let result = p.apply(insert_wind, context);
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events, vec![
        FormulaOpEvent {
            row_idx: 0,
            job_id: job_id("tableId-1", "headerId-1"),
            data: TypedValue::Int(10).get_data(),
            old_data: vec![],
            from: 3,
            action: ActionType::INSERT,
            event_time: events[0].event_time,
        }
    ]);

    let state_opt = context.get_state(&0);
    assert!(state_opt.is_some());
    let state = state_opt.unwrap();
    assert_eq!(state, FormulaState {
        value: TypedValue::Int(10).get_data(),
        node_states: vec![
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 1,
            }
        ],
    });

    let ref insert_wind_delayed = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(800)),
        timestamp: start.add(time::Duration::from_millis(400)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 2,
            },
        ],
    };

    let result_1 = p.apply(insert_wind_delayed, context);
    assert!(result_1.is_ok());
    let events = result_1.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events, vec![
        FormulaOpEvent {
            row_idx: 0,
            job_id: job_id("tableId-1", "headerId-1"),
            data: TypedValue::Int(20).get_data(),
            old_data: TypedValue::Int(10).get_data(),
            from: 3,
            action: ActionType::UPDATE,
            event_time: events[0].event_time,
        }
    ]);

    let state_opt_1 = context.get_state(&0);
    assert!(state_opt_1.is_some());
    let mut state_1 = state_opt_1.unwrap();
    state_1.node_states.sort_by_key(|v| v.node_idx);
    assert_eq!(state_1, FormulaState {
        value: TypedValue::Int(20).get_data(),
        node_states: vec![
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 1,
            },
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 2,
            },
        ],
    })
}

#[test]
fn test_sub_without_values_no_delayed_insert_event_arrival() {
    use stream::pipeline::FormulaOpEventPipeline;
    use stream::window::KeyedWindow;
    use common::types::{formula::FormulaOp, ActionValue, ActionType, TypedValue, FormulaState, ValueState};

    let p = FormulaOpEventPipeline::new(
        FormulaOp::Sub { values: vec![] },
        job_id("tableId-1", "headerId-1"),
        3,
        vec![1, 2],
    );

    let ref context = p.create_context();
    let start = time::SystemTime::now();
    let ref insert_wind = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(400)),
        timestamp: start.add(time::Duration::from_millis(200)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(20),
                from: 1,
            },
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 2,
            },
        ],
    };
    let result = p.apply(insert_wind, context);
    assert!(result.is_ok());
    let events = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events, vec![
        FormulaOpEvent {
            row_idx: 0,
            job_id: job_id("tableId-1", "headerId-1"),
            data: TypedValue::Int(10).get_data(),
            old_data: vec![],
            from: 3,
            action: ActionType::INSERT,
            event_time: events[0].event_time,
        }
    ]);
    let state_opt = context.get_state(&0);
    assert!(state_opt.is_some());
    let mut states = state_opt.unwrap();
    states.node_states.sort_by_key(|v| v.node_idx);
    assert_eq!(states, FormulaState {
        value: TypedValue::Int(10).get_data(),
        node_states: vec![
            ValueState {
                value: TypedValue::Int(20),
                node_idx: 1,
            },
            ValueState {
                value: TypedValue::Int(10),
                node_idx: 2,
            },
        ],
    });
}

#[test]
fn test_mul_without_values_no_delayed_insert_event_arrival() {
    use stream::pipeline::FormulaOpEventPipeline;
    use stream::window::KeyedWindow;
    use common::types::{formula::FormulaOp, ActionValue, ActionType, TypedValue, FormulaState, ValueState};

    let p = FormulaOpEventPipeline::new(
        FormulaOp::Mul { values: vec![] },
        job_id("tableId-1", "headerId-1"),
        3,
        vec![1, 2],
    );

    let ref context = p.create_context();
    let start = time::SystemTime::now();
    let ref insert_wind = KeyedWindow {
        key: 0,
        start,
        end: start.add(time::Duration::from_millis(400)),
        timestamp: start.add(time::Duration::from_millis(200)),
        values: vec![
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(20),
                from: 1,
            },
            ActionValue {
                action: ActionType::INSERT,
                value: TypedValue::Int(10),
                from: 2,
            },
        ],
    };
}
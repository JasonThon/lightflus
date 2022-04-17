use std::ops::Add;
use std::time;
use common::sysenv;
use crate::constants;

#[derive(Clone)]
pub enum TriggerType {
    Watermark
}

pub enum Trigger {
    Watermark {
        firetime: time::SystemTime
    }
}

impl Trigger {
    pub async fn trigger(&mut self) -> bool {
        let now_time = time::SystemTime::now();
        match self {
            Trigger::Watermark { firetime } => now_time >= *firetime
        }
    }
}

impl Default for Trigger {
    fn default() -> Self {
        todo!()
    }
}

impl From<&TriggerType> for Trigger {
    fn from(trigger_type: &TriggerType) -> Self {
        let process_time = time::SystemTime::now();
        match trigger_type {
            TriggerType::Watermark => Self::Watermark {
                firetime: sysenv::get_env(constants::WATERMARK_ENV)
                    .map(|firetime| {
                        process_time.add(time::Duration::from_millis(firetime.parse::<u64>().unwrap()))
                    })
                    .or_else(|| Some(process_time.add(constants::DEFAULT_FIRETIME_DURATION)))
                    .unwrap()
            }
        }
    }
}
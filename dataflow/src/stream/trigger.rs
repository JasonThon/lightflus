use std::ops::{Add, AddAssign};
use std::time;
use common::sysenv;
use crate::constants;

#[derive(Clone)]
pub enum TriggerType {
    Watermark {
        firetime: time::Duration
    }
}

pub enum Trigger {
    Watermark {
        firetime: time::SystemTime,
        mark: time::Duration,
    }
}

impl Trigger {
    pub async fn trigger(&mut self) -> bool {
        let now_time = time::SystemTime::now();
        match self {
            Trigger::Watermark { firetime, .. } => now_time >= *firetime
        }
    }

    pub fn refresh(&mut self) {
        match self {
            Trigger::Watermark { firetime, mark } =>
                firetime.add_assign(mark.clone())
        }
    }
}

impl Default for Trigger {
    fn default() -> Self {
        Self::Watermark {
            firetime: time::SystemTime::now().add(constants::DEFAULT_FIRETIME_DURATION),
            mark: Default::default(),
        }
    }
}

impl From<&TriggerType> for Trigger {
    fn from(trigger_type: &TriggerType) -> Self {
        let process_time = time::SystemTime::now();
        match trigger_type {
            TriggerType::Watermark { firetime } => Self::Watermark {
                firetime: process_time.add(firetime.clone()),
                mark: firetime.clone(),
            }
        }
    }
}
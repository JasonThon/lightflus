use proto::common::{ExecutionId, ResourceId};

pub mod collections;
pub mod db;
pub mod err;
pub mod event;
pub mod kafka;
pub mod net;
pub mod redis;
pub mod types;
pub mod utils;

pub const NANOS_PER_MILLI: u32 = 1_000_000;

/// [`ExecutionID`] is a tuple which contains a copy of values in [`ExecutionId`].
/// Differently, [`ExecutionID`] derives many properties that [`ExecutionId`] does not, like: [`Hash`], [`serde::Serialize`],
/// [`serde::Deserialize`], etc so that [`ExecutionID`] can be used more easily in any module of Lightflus
#[derive(
    Clone,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    Default,
)]
pub struct ExecutionID(pub ResourceId, pub u32);

impl From<&ExecutionId> for ExecutionID {
    fn from(id: &ExecutionId) -> Self {
        Self(id.get_job_id(), id.sub_id)
    }
}

impl ExecutionID {
    pub fn into_prost(&self) -> ExecutionId {
        ExecutionId {
            job_id: Some(self.0.clone()),
            sub_id: self.1,
        }
    }
}

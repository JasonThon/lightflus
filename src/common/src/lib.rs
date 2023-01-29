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

#[cfg(test)]
mod tests {
    use proto::common::ResourceId;

    #[test]
    fn test_resource_id_serialize() {
        let resource_id = ResourceId {
            resource_id: "resource_id".to_string(),
            namespace_id: "namespace_id".to_string(),
        };

        let result = serde_json::to_string(&resource_id);
        assert!(result.is_ok());
        let val = result.unwrap();

        assert_eq!(
            &val,
            "{\"resource_id\":\"resource_id\",\"namespace_id\":\"namespace_id\"}"
        )
    }
}

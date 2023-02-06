pub mod collections;
pub mod db;
pub mod err;
pub mod event;
pub mod kafka;
pub mod net;
pub mod redis;
pub mod types;
pub mod utils;
#[cfg(not(tarpaulin_include))]
pub mod consts;

pub const NANOS_PER_MILLI: u32 = 1_000_000;

#[cfg(test)]
mod tests {
    use proto::common::{ExecutionId, HostAddr, ResourceId};

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

    #[test]
    fn test_resource_id_deserialize() {
        let origin = "{\"resource_id\":\"resource_id\",\"namespace_id\":\"namespace_id\"}";
        let result = serde_json::from_str::<ResourceId>(origin);
        if result.is_err() {
            let err = result.unwrap_err();
            println!("{}", err);
            return;
        }
        assert!(result.is_ok());

        let resource_id = result.unwrap();
        assert_eq!(resource_id.resource_id, "resource_id".to_string());
        assert_eq!(resource_id.namespace_id, "namespace_id".to_string());
    }

    #[test]
    fn test_host_addr_serialize() {
        let host_addr = HostAddr {
            host: "localhost".to_string(),
            port: 1010,
        };

        let result = serde_json::to_string(&host_addr);
        assert!(result.is_ok());
        let val = result.unwrap();

        assert_eq!(&val, "{\"host\":\"localhost\",\"port\":1010}")
    }

    #[test]
    fn test_host_addr_deserialize() {
        let origin = "{\"host\":\"localhost\",\"port\":1010}";
        let result = serde_json::from_str::<HostAddr>(origin);
        if result.is_err() {
            let err = result.unwrap_err();
            println!("{}", err);
            return;
        }
        assert!(result.is_ok());

        let host_addr = result.unwrap();
        assert_eq!(host_addr.host, "localhost".to_string());
        assert_eq!(host_addr.port, 1010);
    }

    #[test]
    fn test_execution_id_serialize() {
        {
            let execution_id = ExecutionId {
                job_id: Some(ResourceId {
                    resource_id: "resource_id".to_string(),
                    namespace_id: "namespace_id".to_string(),
                }),
                sub_id: 111,
            };

            let result = serde_json::to_string(&execution_id);
            assert!(result.is_ok());
            let val = result.unwrap();

            assert_eq!(&val, "{\"job_id\":{\"resource_id\":\"resource_id\",\"namespace_id\":\"namespace_id\"},\"sub_id\":111}");
        }

        {
            let execution_id = ExecutionId {
                job_id: None,
                sub_id: 111,
            };

            let result = serde_json::to_string(&execution_id);
            assert!(result.is_ok());
            let val = result.unwrap();

            assert_eq!(&val, "{\"job_id\":null,\"sub_id\":111}");
        }
    }

    #[test]
    fn test_execution_id_deserialize() {
        {
            let origin = "{\"job_id\":{\"resource_id\":\"resource_id\",\"namespace_id\":\"namespace_id\"},\"sub_id\":111}";
            let result = serde_json::from_str::<ExecutionId>(origin);
            if result.is_err() {
                let err = result.unwrap_err();
                println!("{}", err);
                return;
            }
            assert!(result.is_ok());

            let execution_id = result.unwrap();
            assert_eq!(
                execution_id.job_id,
                Some(ResourceId {
                    resource_id: "resource_id".to_string(),
                    namespace_id: "namespace_id".to_string(),
                })
            );
            assert_eq!(execution_id.sub_id, 111);
        }

        {
            let origin = "{\"job_id\":null,\"sub_id\":111}";
            let result = serde_json::from_str::<ExecutionId>(origin);
            if result.is_err() {
                let err = result.unwrap_err();
                println!("{}", err);
                return;
            }
            assert!(result.is_ok());

            let execution_id = result.unwrap();
            assert_eq!(execution_id.job_id, None);
            assert_eq!(execution_id.sub_id, 111);
        }

        {
            let origin = "{\"sub_id\":111}";
            let result = serde_json::from_str::<ExecutionId>(origin);
            if result.is_err() {
                let err = result.unwrap_err();
                println!("{}", err);
                return;
            }
            assert!(result.is_ok());

            let execution_id = result.unwrap();
            assert_eq!(execution_id.job_id, None);
            assert_eq!(execution_id.sub_id, 111);
        }

        {
            let origin = "{}";
            let result = serde_json::from_str::<ExecutionId>(origin);
            if result.is_err() {
                let err = result.unwrap_err();
                println!("{}", err);
                return;
            }
            assert!(result.is_ok());

            let execution_id = result.unwrap();
            assert_eq!(execution_id.job_id, None);
            assert_eq!(execution_id.sub_id, 0);
        }

        {
            let origin = "{\"job_id\":null}";
            let result = serde_json::from_str::<ExecutionId>(origin);
            if result.is_err() {
                let err = result.unwrap_err();
                println!("{}", err);
                return;
            }
            assert!(result.is_ok());

            let execution_id = result.unwrap();
            assert_eq!(execution_id.job_id, None);
            assert_eq!(execution_id.sub_id, 0);
        }
    }
}

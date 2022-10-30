use proto::common::common::ResourceId;

#[derive(serde::Deserialize)]
pub(crate) struct GetResourceArgs {
    pub resource_type: i32,
    pub resource_id: String,
    pub namespace: String,
}

impl GetResourceArgs {
    pub fn to_resource_id(&self) -> ResourceId {
        let mut resource_id = ResourceId::default();
        resource_id.set_namespace_id(self.namespace.clone());
        resource_id.set_resource_id(self.resource_id.clone());
        resource_id
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ListResourcesArgs {
    pub resource_type: i32,
    pub namespace: String,
}

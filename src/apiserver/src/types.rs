#[derive(serde::Deserialize)]
pub(crate) struct GetResourceArgs {
    pub resource_type: i32,
    pub resource_id: String,
    pub namespace: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct ListResourcesArgs {
    pub resource_type: i32,
    pub namespace: String,
}

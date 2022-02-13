#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DataflowDesc<View> {
    #[serde(rename(serialize = "objectId"))]
    object_id: String,
    view: View,
}
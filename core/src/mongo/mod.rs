use std::intrinsics::unchecked_rem;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MongoDesc {
    pub uri: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl MongoDesc {
    pub async fn to_db(&self) -> mongodb::error::Result<mongodb::Database> {
        mongodb::Client::with_uri_str(self.uri.as_str())
            .await
            .map(|cli| cli.database(self.database.as_str()))
    }
}
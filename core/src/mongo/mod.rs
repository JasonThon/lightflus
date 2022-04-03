use std::fmt::Formatter;
use mongodb::error::{ErrorKind, WriteFailure};

pub fn is_dup_err(err: &mongodb::error::Error) -> bool {
    match err.kind.as_ref() {
        ErrorKind::Write(failure) => match failure {
            WriteFailure::WriteError(err) => err.code == 11000,
            _ => false,
        },
        _ => false
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct MongoConfig {
    pub host: String,
    pub port: usize,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: String,
}

impl core::fmt::Display for MongoConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.uri().as_str())
    }
}

impl MongoConfig {
    fn uri(&self) -> String {
        if self.is_auth() {
            return format!(
                "mongodb://{}:{}@{}:{}/{}",
                &self.username.as_ref().unwrap(),
                &self.password.as_ref().unwrap(),
                &self.host,
                &self.port,
                &self.database
            );
        }

        format!("mongodb://{}:{}/{}", &self.host, &self.port, &self.database)
    }

    pub fn to_client(&self) -> mongodb::error::Result<mongodb::sync::Client> {
        let credential = if self.is_auth() {
            Some(mongodb::options::Credential::builder()
                .username(self.username.as_ref().unwrap().clone())
                .password(self.password.as_ref().unwrap().clone())
                .build())
        } else {
            None
        };

        mongodb::sync::Client::with_options(mongodb::options::ClientOptions::builder()
            .hosts(vec![mongodb::options::ServerAddress::Tcp {
                host: self.host.clone(),
                port: Some(self.port as u16),
            }])
            .default_database(Some(self.database.to_string()))
            .credential(credential)
            .build()
        )
    }

    fn is_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }
}
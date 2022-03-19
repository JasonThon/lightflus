use crate::source::Connector::Redis;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum SourceDesc {
    Redis {
        host: String,
        port: u16,
        username: String,
        password: String,
        db: usize,
    },
    Tableflow {
        host: String,
        port: usize,
        page: u32,
        limit: u32,
    },
}


impl SourceDesc {
    pub fn to_connector(&self) -> Connector {
        match self {
            SourceDesc::Tableflow {
                host, port, page, limit
            } => Connector::Tableflow {
                cli: data_client::new_data_engine_client(
                    data_client::DataEngineConfig {
                        host: host.to_string(),
                        port: port.clone(),
                    }
                ),
                next_page: page.clone(),
                limit: limit.clone(),
            },

            SourceDesc::Redis {
                host, port, username, password, db
            } => Redis {
                cli: redis::Client::open(format!("redis://{}:{}@{}:{}/{}", username, password, host, port, db))?
            }
        }
    }
}

pub enum Connector {
    Tableflow {
        cli: data_client::tableflow_grpc::DataEngineClient,
        next_page: u32,
        limit: u32,
    },

    Redis {
        cli: redis::Client
    },
}
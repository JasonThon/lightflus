use std::borrow::Borrow;
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Response {
    code: u32,
    message: String,
}

impl Response {
    pub fn code(&self) -> u32 {
        self.code.clone()
    }
    pub fn message(&self) -> String {
        self.message.clone()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self)
            .expect("")
    }

    pub fn ok() -> Response {
        Response {
            code: 200,
            message: "success".to_string(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Protocol {
    HTTP,
    HTTPS,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Method {
    Get {
        params: BTreeMap<String, String>
    },
    Post {
        payload: BTreeMap<String, String>
    },
    Delete,
    Patch {
        payload: BTreeMap<String, String>
    },
}


#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpDesc {
    url: String,
    protocol: Protocol,
    path: String,
    method: Method,
    header: BTreeMap<String, String>,
}
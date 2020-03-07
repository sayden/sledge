use hyper::{Body, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::components::errors::Error;
use crate::server::responses::unknown_error;

#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub(crate) error: bool,
    pub(crate) cause: Option<String>,
    pub(crate) data: Option<Box<Value>>,
}

impl From<Reply> for Response<Body> {
    fn from(r: Reply) -> Self {
        let body: String = serde_json::to_string(&r)
            .unwrap_or_else(|err| err.to_string());

        http::Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .map_err(Error::GeneratingResponse)
            .unwrap_or_else(|err| unknown_error(err.to_string()))
    }
}

impl Reply {
    pub fn ok(data: Option<Box<Value>>) -> Self {
        Reply {
            error: false,
            cause: None,
            data,
        }
    }

    pub fn error(err: Error) -> Self {
        Reply {
            error: true,
            cause: Some(err.to_string()),
            data: None,
        }
    }
}
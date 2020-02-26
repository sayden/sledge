
use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone)]
pub struct InsertQueryReq {
    pub(crate) id: Option<String>,
    pub(crate) channel: Option<String>,
}

impl Display for InsertQueryReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or(r#"{"error":true}"#.to_string()))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetReq {
    pub(crate) key: String
}

impl Display for GetReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "key: {}", self.key)
    }
}
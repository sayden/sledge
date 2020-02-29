use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Query {
    pub id: Option<String>,
    pub end: Option<String>,
    pub limit: Option<usize>,
    pub until_key: Option<String>,
    pub skip: Option<usize>,
    pub direction_forward: Option<bool>,
    pub channel: Option<String>,
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}\nend: {}\nlimit: {}\nuntil_key: {}\nskip: {}\ndirection_forward: {}\nchannel: {}\n",
               self.id.clone().unwrap_or_default(),
               self.end.clone().unwrap_or_default(),
               self.limit.clone().unwrap_or_default(),
               self.until_key.clone().unwrap_or_default(),
               self.skip.clone().unwrap_or_default(),
               self.direction_forward.clone().unwrap_or_default(),
               self.channel.clone().unwrap_or_default())
    }
}

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
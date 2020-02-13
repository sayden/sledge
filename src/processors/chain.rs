use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use anyhow::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Modifier {
    #[serde(rename = "type")]
    pub type_: String,
    pub field: String,
}

pub trait ModifierTrait {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<Error>;
}
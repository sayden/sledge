use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Stats {
    pub total_entries: u128,
    pub error: String,
}

impl ToString for Stats {
    fn to_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(s) => s,
            Err(err) => err.to_string(),
        }
    }
}
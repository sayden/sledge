use crate::components::storage::Storage;
use crate::storage::stats::Stats;

pub struct V1 {
    pub storage: Box<dyn Storage + Send + Sync>
}

pub fn new(f: Box<dyn Storage + Send + Sync>) -> V1 {
    return V1 { storage: f };
}

impl V1 {
    pub fn get_by_id(&self, k: String) -> Result<Option<String>, anyhow::Error> {
        self.storage.get(k)
    }

    pub fn stats(&self) -> Stats {
        self.storage.stats()
    }

    pub fn new(s: Box<dyn Storage + Send + Sync>) -> Self {
        V1{
            storage: s,
        }
    }
}

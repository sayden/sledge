use anyhow::Error;
use crate::components::storage::Storage;
use crate::storage::stats::Stats;

pub struct V1 {
    pub storage: Box<dyn Storage>
}

pub fn new(f: Box<dyn Storage>) -> V1 {
    return V1 { storage: f };
}

impl V1 {
    fn get_by_id(&self, k: String) -> Result<Option<String>, anyhow::Error> {
        self.storage.get(k)
    }

    pub(crate) fn stats(&self) -> Stats {
        self.storage.stats()
    }
}

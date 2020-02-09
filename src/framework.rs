use crate::storage::Storage;
use crate::app_errors::{AppErrorV2};

pub trait Framework {
    fn get(&self, k: &str) -> Result<Option<String>, AppErrorV2>;
    fn put(&self, k: &str, v: &str) -> Result<Option<String>, AppErrorV2>;
}

pub struct FrameworkV1 {
    pub storage: Box<dyn Storage>
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<Option<String>, AppErrorV2> {
        self.storage.get(s)
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, AppErrorV2> {
        self.storage.put(k,v)
    }
}

pub fn new(s: Box<dyn Storage>) -> Box<dyn Framework> {
    Box::new(FrameworkV1 { storage: s })
}
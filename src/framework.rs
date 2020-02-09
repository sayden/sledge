use crate::storage::Storage;
use crate::errors::AppError;

pub trait Framework {
    fn get(&self, k: &str) -> Result<Option<String>, AppError>;
    fn put(&self, k: &str, v: &str) -> Result<(), AppError>;
}

pub struct FrameworkV1 {
    pub storage: Box<dyn Storage>
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<Option<String>, AppError> {
        self.storage.get(s)
    }
    fn put(&self, k: &str, v: &str) -> Result<(), AppError> {
        self.storage.put(k, v)
    }
}

pub fn new(s: Box<dyn Storage>) -> Box<dyn Framework> {
    Box::new(FrameworkV1 { storage: s })
}

pub struct KV {
    k: String,
    v: String,
}
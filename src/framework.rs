use crate::storage::Storage;
use crate::errors::AppError;

pub trait Framework {
    fn get(&self, k: &str) -> Result<Option<String>, AppError>;
    fn put(&self, k: &str, v: &str) -> Result<(), AppError>;
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error>;
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
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        self.storage.range(k)
    }
}

pub fn new(s: Box<dyn Storage>) -> Box<dyn Framework> {
    Box::new(FrameworkV1 { storage: s })
}
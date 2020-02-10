use crate::components::Storage;

pub trait Framework {
    fn get(&self, k: &str) -> Result<Option<String>, failure::Error>;
    fn put(&self, k: &str, v: &str) -> Result<(), failure::Error>;
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error>;
}

pub struct FrameworkV1 {
    pub storage: Box<dyn Storage>
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<Option<String>, failure::Error> {
        self.storage.get(s)
    }
    fn put(&self, k: &str, v: &str) -> Result<(), failure::Error> {
        self.storage.put(k, v)
    }

    /**
    * Types of range operations:
    * (window) since upper bounded with a limit based on count
    * (window) since upper bounded with a limit on a key found (or limit)
    * since infinite, no stop if possible
    * since infinite until signal
    * backwards from key down to a limit based on count
    * backwards from key down to a key found (or limit)
    * backwards from key, infinite
    * backwards from key, infinite until signal
    */
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        self.storage.range(k)
    }
}

pub fn new(s: Box<dyn Storage>) -> Box<dyn Framework> {
    Box::new(FrameworkV1 { storage: s })
}
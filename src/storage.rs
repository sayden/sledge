pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, failure::Error>;
    fn put(&self, k: &str, v: &str) -> Result<(), failure::Error>;
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error>;
}
pub trait StorageV1 {
    fn get(&self, s: &str) -> Result<String, String>;
    fn put(&self, k: &str, v: &str) -> Result<String, String>;
}
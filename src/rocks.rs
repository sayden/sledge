struct RocksDB {
    db: rocks,
}

impl StorageV1 for RocksDB {
    fn get(&self, s: &str) -> Result<String, String> {
        let result = self.db.get(s);
        Ok(String::from_utf8(result?.unwrap()).unwrap())
    }

    fn put(&self, k: &str, v: &str) -> Result<String, String> {
        unimplemented!()
    }
}
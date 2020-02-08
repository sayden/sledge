struct Sled {
    db: SledDb,
}

impl Sled {
    fn new(p: &str) -> Self {
        let db = sled::open(p).unwrap();
        Sled { db }
    }

    fn ivec_to_string(ivec: IVec)-> String {
        String::from_utf8(ivec.to_vec()).unwrap()
    }
}

impl StorageV1 for Sled {
    fn get(&self, s: &str) -> Result<String, String> {
        let result = self.db.get(s);
        Ok(Sled::ivec_to_string(result.unwrap().unwrap()))
    }

    fn put(&self, k: &str, v: &str) -> Result<String, String> {
        Ok(Sled::ivec_to_string(self.db.insert(k,v).unwrap().unwrap()))
    }
}
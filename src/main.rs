use sled::{Db as SledDb, IVec};
use rocksdb::DB as rocks;

fn main() {
    let sdb = Sled::new("/tmp/sled");
    println!("{}", sdb.put("hello", "world").unwrap());
    println!("{}", sdb.get("hello").unwrap());
}

pub trait App {
    fn get_by_id(&self, s: &str) -> Result<String, String>;
}

struct V1 {
    framework: FrameworkV1
}

impl App for V1 {
    fn get_by_id(&self, s: &str) -> Result<String, String> {
        unimplemented!()
    }
}

pub trait Framework {
    fn get(&self, s: &str) -> Result<String, String>;
}

struct FrameworkV1 {
    storage: dyn StorageV1
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<String, String> {
        unimplemented!()
    }
}

pub trait StorageV1 {
    fn get(&self, s: &str) -> Result<String, String>;
    fn put(&self, k: &str, v: &str) -> Result<String, String>;
}

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
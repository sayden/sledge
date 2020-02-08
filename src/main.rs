use sled::{Db as SledDb, IVec};
use rocksdb::DB as rocks;

fn main() {
    let sdb = Sled::new("/tmp/sled");
    println!("insertion: {}", sdb.put("hello", "world").unwrap());
    println!("retrieval {}", sdb.get("hello").unwrap());
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
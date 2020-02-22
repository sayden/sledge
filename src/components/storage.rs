use anyhow::Error;
use crate::storage::sled::Sled;
use crate::storage::rocks::Rocks;
use crate::storage::void::Void;
use crate::storage::memory::Memory;
use crate::components::kv::KV;
use crate::storage::stats::Stats;

pub type SledgeIterator = dyn Iterator<Item=KV>;

/**
* Types of range operations:
* (window) since upper bounded with a limit based on count
* (window) since upper bounded with a limit on a key found (or limit)
* since: infinite, no stop if possible
* since: infinite until signal
* backwards from key down to a limit based on count
* backwards from key down to a key found (or limit)
* backwards from key, infinite
* backwards from key, infinite until signal
*
* Examples:
* since("my_key", Bound::Infinite)
* since("my_key", Bound::Limit(100))
* since("my_key", Bound::Key("stop_in_this_key"))
* since("my_key", Bound::Key("stop_in_this_key"), Bound::Limit(10))
* since("my_key", Bound::Key("stop_in_this_key"), Bound::KV(KV{key:"stop_if_this_key",value:"has_this_value"))
*
* backwards("my_key", Bound::Infinite)
* backwards("my_key", Bound::Limit(100))
* backwards("my_key", Bound::Key("stop_in_this_key"))
* backwards("my_key", Bound::Key("stop_in_this_key"), Bound::Limit(10))
* backwards("my_key", Bound::Key("stop_in_this_key"), Bound::KV(KV{key:"stop_if_this_key",value:"has_this_value"))
*/
pub trait Storage {
    fn get(&self, s: String) -> Result<Option<String>, Error>;
    fn put(&mut self, k: String, v: String) -> Result<(), Error>;

    fn start(&self) -> Result<Box<SledgeIterator>, Error>;
    fn end(&self) -> Result<Box<SledgeIterator>, Error>;

    fn since(&self, k: String) -> Result<Box<SledgeIterator>, Error>;
    fn since_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error>;

    fn reverse(&self, k: String) -> Result<Box<SledgeIterator>, Error>;
    fn reverse_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error>;

    fn stats(&self) -> Stats;
}

pub fn get_storage(s: &str, p: &str) -> Box<dyn Storage + Send + Sync>  {
    match s {
        "sled" => Sled::new(p.to_string()),
        "rocksdb" => Rocks::new(p.to_string()),
        "void" => Void::new(),
        "memory" => Memory::new(),
        _ => panic!("storage '{}' not found", s),
    }
}

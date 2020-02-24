use crate::storage::sled::Sled;
use crate::storage::rocks::Rocks;
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::str::Utf8Error;


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
    fn get(&self, keyspace: Option<String>, s: String) -> Result<Option<String>, Error>;
    fn put(&mut self, keyspace: Option<String>, k: String, v: String) -> Result<(), Error>;
    fn create_keyspace(&mut self, name: String) -> Result<(), Error>;

    fn start<'a>(&'a self, keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn end<'a>(&'a self, keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn since<'a>(&'a self, keyspace: Option<String>, k: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn since_until<'a>(&'a self, keyspace: Option<String>, k1: String, k2: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn reverse<'a>(&'a self, keyspace: Option<String>, k: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn reverse_until<'a>(&'a self, keyspace: Option<String>, k1: String, k2: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn stats(&self) -> Stats;
}

pub fn get_storage(s: &str, p: &str) -> Box<dyn Storage + Send + Sync> {
    match s {
        "sled" => Sled::new(p.to_string()),
        "rocksdb" => Rocks::new(p.to_string()),
        // "memory" => Memory::new(),
        _ => panic!("storage '{}' not found", s),
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Get(String),

    #[error("{0}")]
    Iterator(String),

    #[error("{0}")]
    ValueNotFound(String),

    #[error(transparent)]
    Parse(#[from] Utf8Error),

    #[error("error preparing op: {0}")]
    Preparing(String),

    #[error("error returned from db: {0}")]
    Db(String),

    #[error("error doing put {0}")]
    Put(String),

    #[error("error creating keyspace with name {0}: {1}")]
    CannotCreateKeyspace(String, String),

    #[error("keyspace with name {0} not found")]
    NotFound(String),

    #[error("error opening keyspace with name {0} not found")]
    Open(String),

    #[error("keyspace error with name '{0}'")]
    Keyspace(String)
}
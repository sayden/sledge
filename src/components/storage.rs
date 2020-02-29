use crate::storage::sled::Sled;
use crate::storage::rocks::Rocks;
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use bytes::Bytes;
use crate::server::requests::Query;

pub type SledgeIterator = dyn Iterator<Item=KV>;

pub enum IterMod {
    Skip(usize),
    Limit(usize),
    UntilKey(String),
}

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
*/
pub trait Storage {
    fn get(&self, keyspace: Option<String>, s: String) -> Result<String, Error>;
    fn put(&mut self, keyspace: Option<String>, k: String, v: Bytes) -> Result<(), Error>;
    fn create_keyspace(&mut self, name: String) -> Result<(), Error>;

    fn start<'a>(&'a self, keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn end<'a>(&'a self, keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn range<'a>(&'a self, keyspace: Option<String>, query: Query) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn since<'a>(&'a self, keyspace: Option<String>, k: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn since_until<'a>(&'a self, keyspace: Option<String>, k1: String, k2: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn reverse<'a>(&'a self, keyspace: Option<String>, k: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;
    fn reverse_until<'a>(&'a self, keyspace: Option<String>, k1: String, k2: String) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error>;

    fn stats(&self) -> Stats;
}

pub fn get_storage(s: &str, p: &str) -> Box<dyn Storage + Send + Sync> {
    match s {
        "sled" => Box::new(Sled::new(p.to_string())),
        "rocksdb" => Box::new(Rocks::new(p.to_string())),
        // "memory" => Memory::new(),
        _ => panic!("storage '{}' not found", s),
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error doing get: {0}")]
    Get(String),

    #[error("iterator error: {0}")]
    Iterator(String),

    #[error("value not found: {0}")]
    ValueNotFound(String),

    #[error(transparent)]
    Parse(#[from] Utf8Error),

    #[error(transparent)]
    ParseFromUtf8(#[from] FromUtf8Error),

    #[error("error preparing op: {0}")]
    Preparing(String),

    #[error("error returned from db: {0}")]
    Db(String),

    #[error("error doing put {0}")]
    Put(String),

    #[error("error creating keyspace with name {0}: {1}")]
    CannotCreateKeyspace(String, String),

    #[error("cannot retrieve cf with name {0}")]
    CannotRetrieveCF(String),

    #[error("keyspace with name {0} not found")]
    NotFound(String),

    #[error("error serializing data: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("error serializing data: {0}")]
    SerializationString(String),

    #[error("id not found in query")]
    WrongQuery,

}

pub fn put_error(cause: String) -> Result<(), Error> {
    Err(Error::Put(cause))
}

pub fn create_keyspace_error(name: String, cause: String) -> Result<(), Error> {
    Err(Error::CannotCreateKeyspace(name, cause))
}
// use crate::storage::sled::Sled;
use bytes::Bytes;
use crate::components::kv::KV;
use crate::server::requests::Query;
use crate::storage::rocks::Rocks;
use crate::storage::stats::Stats;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

pub enum IterMod {
    Skip(usize),
    Limit(usize),
    UntilKey(String),
}

pub type StorageIter<'a> = Box<dyn Iterator<Item=KV> + Send + Sync + 'a>;
pub type VecIter<'a> = Box<dyn Iterator<Item=Vec<u8>> + 'a + Send + Sync>;
pub type ConcurrentStorage = Box<dyn Storage + Send + Sync>;

pub trait Storage {
    fn get(&self, keyspace: Option<String>, s: String) -> Result<String, Error>;
    fn put(&mut self, keyspace: Option<String>, k: String, v: Bytes) -> Result<(), Error>;
    fn create_keyspace(&mut self, name: String) -> Result<(), Error>;

    fn start(&self, keyspace: Option<String>) -> Result<StorageIter, Error>;
    fn end(&self, keyspace: Option<String>) -> Result<StorageIter, Error>;

    fn range(&self, keyspace: Option<String>, query: Query) -> Result<StorageIter, Error>;

    fn since(&self, keyspace: Option<String>, k: String) -> Result<StorageIter, Error>;
    fn since_until(&self, keyspace: Option<String>, k1: String, k2: String) -> Result<StorageIter, Error>;

    fn reverse(&self, keyspace: Option<String>, k: String) -> Result<StorageIter, Error>;
    fn reverse_until(&self, keyspace: Option<String>, k1: String, k2: String) -> Result<StorageIter, Error>;

    fn stats(&self) -> Stats;
}

pub fn get_storage(s: &str, p: &str) -> Box<dyn Storage + Send + Sync> {
    match s {
        // "sled" => box Sled::new_storage(p.to_string()),
        "rocksdb" => box Rocks::new_storage(p.to_string()),
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

    #[error("mutator list in channel is empty")]
    EmptyMutator,

}

pub fn put_error(cause: String) -> Result<(), Error> {
    Err(Error::Put(cause))
}

pub fn create_keyspace_error(name: String, cause: String) -> Result<(), Error> {
    Err(Error::CannotCreateKeyspace(name, cause))
}
use bytes::Bytes;
use crate::components::kv::KV;
use crate::server::query::Query;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use futures::Stream;

pub enum IterMod {
    Skip(usize),
    Limit(usize),
    UntilKey(String),
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

    #[error("error getting channel")]
    ChannelError,

}

pub fn put_error(cause: String) -> Result<(), Error> {
    Err(Error::Put(cause))
}

pub fn create_keyspace_error(name: String, cause: String) -> Result<(), Error> {
    Err(Error::CannotCreateKeyspace(name, cause))
}
use std::str::Utf8Error;
use std::string::FromUtf8Error;

pub enum IterMod {
    Skip(usize),
    Limit(usize),
    UntilKey(String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error<'a> {
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

    #[error("column family '{0}' not found")]
    CFNotFound(&'a str),

    #[error("keyspace with name {0} not found")]
    NotFound(&'a str),

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

pub fn create_keyspace_error<'a>(name: String, cause: String) -> Result<(), Error<'a>> {
    Err(Error::CannotCreateKeyspace(name, cause))
}
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use hyper::{Body, Response};

use crate::server::reply::Reply;
use crate::server::responses::unknown_error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error parsing HTTP body: {0}")]
    BodyParsingError(#[from] hyper::Error),
    
    #[error("error {0}")]
    Multi(String),

    #[error("error applying filters")]
    FilterError,

    #[error("id with key '{0}' was not found in the incoming json")]
    IdNotFoundInJSON(String),

    #[error("no id found on request")]
    NoIdFoundOnRequest,

    #[error("an ID is required")]
    MissingID,

    #[error("a query is required")]
    MissingQuery,

    #[error("rocksdb error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("kafka error: {0}")]
    KafkaError(#[from] rdkafka::error::KafkaError),

    #[error("value not found: {0}")]
    ValueNotFound(String),

    #[error(transparent)]
    Parse(#[from] Utf8Error),

    #[error(transparent)]
    ParseFromUtf8(#[from] FromUtf8Error),

    #[error("utf8 parsing error: {0}")]
    Utf8Error(String),

    #[error("error preparing op: {0}")]
    Preparing(String),

    #[error("error returned from db: {0}")]
    Db(String),

    #[error("error doing put {0}")]
    Put(String),

    #[error("error creating db with name {0}: {1}")]
    CannotCreateDb(String, String),

    #[error("error reading db with name {0}: {1}")]
    CannotReadDB(String, String),

    #[error("cannot retrieve cf with name {0}")]
    CannotRetrieveCF(String),

    #[error("column family '{0}' not found")]
    CFNotFound(String),

    #[error("id/db '{0}' not found")]
    NotFound(String),

    #[error("channel '{0}' not found")]
    ChannelNotFound(String),

    #[error("json error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("error (des)serializing data: {0}")]
    Serializing(String),

    #[error("id not found in query")]
    WrongQuery,

    #[error("error applying channel: {0}")]
    ChannelError(String),

    #[error("method not implemented")]
    MethodNotFound,

    #[error("error generating response: {0}")]
    GeneratingResponse(#[from] http::Error),

    #[error("error parsing sql: {0}")]
    SqlError(#[from] sqlparser::parser::ParserError),
}

impl From<Error> for Response<Body> {
    fn from(err: Error) -> Self {
        let string = match serde_json::to_string(&Reply::error(err)) {
            Ok(s) => s,
            Err(err) => err.to_string(),
        };

        let res = http::Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(string));

        match res {
            Ok(ok_res) => ok_res,
            Err(err) => unknown_error(err.to_string()),
        }
    }
}

use std::convert::Infallible;
use serde_json::Value;
use crate::server::responses::{ReadReply, ResultEmbeddedReply, WriteReply};

pub(crate) fn new_read(cause: String, db: Option<String>) -> Result<ReadReply, Infallible> {
    Ok(ReadReply {
        result: ResultEmbeddedReply {
            error: true,
            cause: Some(cause),
            db,
        },
        data: None,
    })
}

pub(crate) fn new_write(cause: &str, db: Option<String>) -> Result<WriteReply, Infallible> {
    Ok(WriteReply {
        result: ResultEmbeddedReply {
            error: true,
            cause: Some(cause.to_string()),
            db,
        },
        id: None,
    })
}
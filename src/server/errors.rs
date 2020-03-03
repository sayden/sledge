use std::convert::Infallible;

use crate::server::responses::{ReadReply, ResultEmbeddedReply, WriteReply};

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

pub(crate) fn new_write_string(cause: String, db: Option<String>) -> Result<WriteReply, Infallible> {
    return new_write(cause.as_str(), db);
}
use std::convert::Infallible;

use crate::server::responses::{ReadReply, ResultEmbeddedReply, WriteReply};
use warp::Reply;

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

pub(crate) fn new_boxed_read(cause: String, db: Option<String>) -> Result<Box<dyn Reply>, Infallible> {
    Ok(Box::new(ReadReply {
        result: ResultEmbeddedReply {
            error: true,
            cause: Some(cause),
            db,
        },
        data: None,
    }))
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
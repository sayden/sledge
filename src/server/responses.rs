use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::convert::Infallible;

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultEmbeddedReply {
    pub(crate)error: bool,
    pub(crate)cause: Option<String>,
    pub(crate)db: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReadReply {
    pub(crate)result: ResultEmbeddedReply,
    pub(crate)data: Option<Box<[Value]>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WriteReply {
    pub(crate)result: ResultEmbeddedReply,
    pub(crate)id: Option<String>,
}

pub(crate) fn new_write(id: Option<String>, db: Option<String>) -> Result<WriteReply, Infallible> {
    Ok(WriteReply {
        result: ResultEmbeddedReply {
            error: false,
            cause: None,
            db,
        },
        id,
    })
}
use warp::reply::Response;
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

impl warp::Reply for ReadReply {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string(&self).unwrap().into())
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WriteReply {
    pub(crate)result: ResultEmbeddedReply,
    pub(crate)id: Option<String>,
}

impl warp::Reply for WriteReply {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string(&self).unwrap().into())
    }
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

pub(crate) fn new_read(data: Option<Box<[Value]>>, db: Option<String>) -> Result<ReadReply, Infallible> {
    Ok(ReadReply {
        result: ResultEmbeddedReply {
            error: false,
            cause: None,
            db,
        },
        data,
    })
}
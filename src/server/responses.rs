use hyper::Body;
use hyper::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::components::errors::Error;

#[derive(Serialize, Deserialize)]
pub struct RangeResult {
    id: String,
    data: Box<Value>,
}

pub fn range_result_to_string(rr: &RangeResult) -> Option<String> {
    serde_json::to_string(rr)
        .map_err(|err| log::warn!("error parsing range result {}", err.to_string()))
        .ok()
}

pub fn new_range_result_string(id: &[u8], data: &[u8]) -> Option<String> {
    range_result_to_string(&new_range_result(id, data)?)
    // serde_json::to_string(&new_range_result(id, data)?)
    //     .map_err(|err| log::warn!("error parsing range result {}", err.to_string()))
    //     .ok()
}

pub fn new_range_result(id: &[u8], data: &[u8]) -> Option<RangeResult> {
    let id_ = std::str::from_utf8(id).unwrap_or_default();

    let data: Box<Value> = box match serde_json::from_slice(data) {
        Ok(res) => res,
        Err(err) => {
            log::warn!("error parsing result data {}", err.to_string());
            return None;
        }
    };

    Some(RangeResult { id: id_.into(), data })
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultEmbeddedReply<C: ToString> {
    pub(crate)error: bool,
    pub(crate)cause: Option<C>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReadReply<C: ToString> {
    pub(crate)result: ResultEmbeddedReply<C>,
    pub(crate)data: Option<Box<Value>>,
    pub(crate)id: Option<C>,
}

impl<C: ToString> ResultEmbeddedReply<C> {
    pub fn ok() -> Self {
        ResultEmbeddedReply {
            error: false,
            cause: None,
        }
    }

    pub fn error(err: C) -> Self {
        ResultEmbeddedReply {
            error: true,
            cause: Some(err),
        }
    }
}

impl<C: ToString + Serialize> From<ReadReply<C>> for Response<Body> {
    fn from(r: ReadReply<C>) -> Self {
        response_from_body(serde_json::to_string(&r)
            .unwrap_or_else(|err| err.to_string()))
            .unwrap_or_else(|err| unknown_error(err.to_string()))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ErrorReply<C: ToString> {
    pub(crate)result: ResultEmbeddedReply<C>,
}

pub fn new_read_ok<'a>(res: &[u8], id: &str) -> Result<Response<Body>, Error> {
    let data: Box<Value> = box serde_json::from_slice(res)
        .map_err(Error::SerdeError)?;

    let reply = ReadReply::<&str> {
        result: ResultEmbeddedReply::ok(),
        data: Some(data),
        id: Some(id),
    };

    Ok(reply.into())
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WriteReply<'a, C: ToString> {
    pub(crate)result: ResultEmbeddedReply<C>,
    pub(crate)id: Option<&'a str>,
}

impl<'a, C: ToString + Serialize> From<WriteReply<'a, C>> for Response<Body> {
    fn from(r: WriteReply<'a, C>) -> Self {
        response_from_body(serde_json::to_string(&r)
            .unwrap_or_else(|err| err.to_string()))
            .unwrap_or_else(|err| unknown_error(err.to_string()))
    }
}

pub fn response_from_body<'a>(body: String) -> Result<Response<Body>, Error> {
    http::Response::builder()
        .header(
            "Content-Type",
            "application/json",
        )
        .body(Body::from(body))
        .map_err(Error::GeneratingResponse)
}

pub fn unknown_error(err: String) -> Response<Body> {
    http::Response::builder()
        .header(
            "Content-Type",
            "application/json",
        )
        .body(Body::from(
            format!(r#"{{"result":{{"error":"true", "cause":"{}"}}}}"#, err)))
        .unwrap()
}
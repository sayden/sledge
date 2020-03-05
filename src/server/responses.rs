use hyper::Body;
use hyper::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct RangeResult {
    id: String,
    data: Box<Value>,
}

pub fn new_range_result(id: &[u8], data: &[u8]) -> Option<String> {
    let id_ = std::str::from_utf8(id).unwrap_or_default();

    let data: Box<Value> = box match serde_json::from_slice(data) {
        Ok(res) => res,
        Err(err) => {
            log::warn!("error parsing result data {}", err.to_string());
            return None;
        }
    };

    serde_json::to_string(&RangeResult { id: id_.into(), data }).ok()
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ResultEmbeddedReply<C: ToString, D: Serialize> {
    pub(crate)error: bool,
    pub(crate)cause: Option<C>,
    pub(crate)db: Option<D>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReadReply<C: ToString, Db: Serialize> {
    pub(crate)result: ResultEmbeddedReply<C, Db>,
    pub(crate)data: Option<Box<Value>>,
    pub(crate)id: Option<C>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ErrorReply<C: ToString, Db: Serialize> {
    pub(crate)result: ResultEmbeddedReply<C, Db>,
}

pub fn new_read_ok(res: &[u8], id: &str, db: &str) -> Response<Body> {
    let data: Box<Value> = box match serde_json::from_slice(res) {
        Ok(res) => res,
        Err(err) => return new_error(err, Some(&db)),
    };

    response_from_body(match serde_json::to_string(&ReadReply::<&str, &str> {
        result: ResultEmbeddedReply {
            error: false,
            cause: None,
            db: Some(db),
        },
        data: Some(data),
        id: Some(id),
    }) {
        Ok(s) => s,
        Err(err) => err.to_string(),
    })
}

pub fn new_error<C: ToString>(cause: C, db: Option<&str>) -> Response<Body> {
    response_from_body(match serde_json::to_string(&ErrorReply::<&str, &str> {
        result: ResultEmbeddedReply {
            error: true,
            cause: Some(&cause.to_string()),
            db,
        },
    }) {
        Ok(s) => s,
        Err(err) => err.to_string(),
    })
}


#[derive(Serialize, Deserialize)]
pub(crate) struct WriteReply<'a, C: ToString, D: Serialize> {
    pub(crate)result: ResultEmbeddedReply<C, D>,
    pub(crate)id: Option<&'a str>,
}

pub fn new_write_ok(id: &str, db: &str) -> Response<Body> {
    response_from_body(match serde_json::to_string(&WriteReply::<&str, &str> {
        result: ResultEmbeddedReply {
            error: false,
            cause: None,
            db: Some(db),
        },
        id: Some(id),
    }) {
        Ok(s) => s,
        Err(err) => err.to_string(),
    })
}

fn response_from_body(body: String) -> Response<Body> {
    http::Response::builder()
        .header(
            "Content-Type",
            "application/json",
        )
        .body(Body::from(body))
        .unwrap_or_else(|err| http::Response::new(Body::from(err.to_string())))
}


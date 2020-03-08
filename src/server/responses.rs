use bytes::Bytes;
use hyper::Body;
use hyper::Response;
use serde_json::Value;

use crate::components::errors::Error;
use crate::components::iterator::SledgeIterator;
use crate::components::simple_pair::{simple_pair_to_json};
use crate::server::handlers::{BytesResultIterator, BytesResultStream};
use crate::server::query::Query;
use crate::server::reply::Reply;

pub fn new_read_ok<'a>(res: &[u8], id: Option<&str>) -> Result<Response<Body>, Error> {
    let data: Box<Value> = box serde_json::from_slice(res)
        .map_err(Error::SerdeError)?;
    let reply = Reply::ok(Some(data));
    Ok(reply.into())
}

pub fn new_read_ok_iter<'a>(iter: SledgeIterator) -> Result<Response<Body>, Error> {
    let data = box serde_json::to_value(iter
        .flat_map(|x| simple_pair_to_json(x, true))
        .collect::<Vec<Value>>())
        .map_err(Error::SerdeError)?;

    let reply = Reply::ok(Some(data));

    Ok(reply.into())
}

pub fn get_iterating_response(iter: SledgeIterator, query: Option<Query>) -> Result<Response<Body>, Error> {
    let include_id = query.and_then(|q| q.include_ids).unwrap_or_else(|| false);

    let thread_iter: Box<BytesResultIterator> = box iter
        .flat_map(move |x| simple_pair_to_json(x, include_id))
        .flat_map(|spj| serde_json::to_string(&spj)
            .map_err(|err| log::warn!("error trying to get json from simpleJSON: {}", err.to_string()))
            .ok())
        .map(|s| format!("{}\n", s))
        .map(|x| Ok(Bytes::from(x)));

    let stream: BytesResultStream = box futures::stream::iter(thread_iter);

    http::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(stream))
        .map_err(Error::GeneratingResponse)
}

pub fn unknown_error(err: String) -> Response<Body> {
    http::Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(format!(
            r#"{{"result":{{"error":"true", "cause":"{}"}}}}"#,
            err
        )))
        .unwrap()
}

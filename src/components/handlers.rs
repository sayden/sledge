use std::convert::Infallible;

use futures::{Stream, StreamExt};
use futures_util::future;
use http::Response;
use hyper::Body;

use crate::channels::parser::Channel;
use crate::components::rocks;
use crate::components::storage::Error;
use crate::server::{errors, responses};
use crate::server::query::Query;

pub async fn put(cf: String, maybe_query: Option<Query>, maybe_path_id: Option<&str>, req: Body)
                 -> Result<Response<Body>, Infallible>
{
    let whole_body = match hyper::body::to_bytes(req).await {
        Err(err) => return Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/json",
            )
            .body(Body::from(r#"{"error": "true"}"#))
            .unwrap()),
        Ok(body) => body,
    };

    let id = rocks::get_id(&maybe_query, maybe_path_id, Some(&whole_body));
    return match id {
        Some(id) => {
            match rocks::put(cf, id, whole_body) {
                Ok(()) => Ok(http::Response::builder()
                    .header(
                        "Content-Type",
                        "application/json",
                    )
                    .body(Body::from(r#"{"error": "false"}"#))
                    .unwrap()),
                Err(err) => Ok(http::Response::builder()
                    .header(
                        "Content-Type",
                        "application/json",
                    )
                    .body(Body::from(r#"{"error": "true"}"#))
                    .unwrap()),
            }
        }
        None => Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/json",
            )
            .body(Body::from(r#"{"error": "true"}"#))
            .unwrap()),
    }
}


pub fn get(keyspace: String, k: String) -> Result<Response<Body>, Infallible> {
    match rocks::get(keyspace, k) {
        Ok(res) => Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/octet-stream",
            )
            .body(Body::from(res))
            .unwrap()),
        Err(err) => Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/json",
            )
            .body(Body::from(r#"{"error": "true"}"#))
            .unwrap())
    }
}

fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error>
{
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            Some(channel_id) => {
                let res = rocks::get("_channel".to_string(), channel_id.clone())?;
                let c = Channel::new(res.as_str())?;
                return Ok(Some(c));
            }
            None => Ok(None),
        }
    }
}

fn error_response(cause: String) -> Response<Body> {
    http::Response::builder()
        .header(
            "Content-Type",
            "application/json",
        )
        .body(Body::from(r#"{"error": "true"}"#))
        .unwrap()
}
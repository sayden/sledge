use std::convert::Infallible;

use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use uuid::Uuid;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::rocks;
use crate::components::rocks::{StreamItem};
use crate::components::storage::Error;
use crate::server::query::Query;
use crate::server::responses::{new_read_error, new_read_ok, new_write_error, new_write_ok};

pub async fn range(maybe_query: Option<Query>, maybe_path_id: Option<&str>, cf_name: &str, maybe_channel: Option<Channel>) -> Result<Response<Body>, Infallible> {
    let maybe_id = get_id(&maybe_query, maybe_path_id, None);

    let thread_iter = match rocks::range(maybe_query, maybe_id, cf_name, maybe_channel) {
        Ok(iter) => iter,
        Err(err) => return Ok(err),
    };

    let stream: Box<dyn Stream<Item=StreamItem> + Send + Sync> = box futures::stream::iter(thread_iter);

    let response = http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream))
        .unwrap();

    Ok(response)
}

pub async fn put(cf: &str, maybe_query: Option<Query>, maybe_path_id: Option<&str>, req: Body, maybe_channel: Option<Channel>)
                 -> Result<Response<Body>, Infallible>
{
    let whole_body = match hyper::body::to_bytes(req).await {
        Err(err) => return Ok(new_write_error(err, None, cf)),
        Ok(body) => body,
    };

    let id = match get_id(&maybe_query, maybe_path_id, Some(&whole_body)) {
        Some(id) => id,
        None => return Ok(new_write_error("no id found", None, cf))
    };

    let maybe_data = match pass_through_channel(maybe_query, whole_body.as_ref(), maybe_channel) {
        Ok(d) => d,
        Err(response) => return response,
    };

    let data = maybe_data.unwrap_or(whole_body);

    match rocks::put(cf, &id, data) {
        Ok(()) => Ok(new_write_ok(id, cf)),
        Err(err) => Ok(new_write_error(err, Some(id), cf)),
    }
}

pub fn get(maybe_query: Option<Query>, cf: String, id: String, maybe_channel: Option<Channel>) -> Result<Response<Body>, Infallible> {
    let value = match rocks::get(&cf, &id) {
        Ok(res) => res,
        Err(err) => return Ok(new_read_error(err, id.into(), cf.into()))
    };

    let data = match pass_through_channel(maybe_query, value.as_ref(), maybe_channel) {
        Ok(d) => d,
        Err(response) => return response,
    };

    Ok(new_read_ok(data.unwrap_or(Bytes::from(value)).as_ref(), id, cf))
}

fn pass_through_channel(maybe_query: Option<Query>, whole_body: &[u8], maybe_channel: Option<Channel>) -> Result<Option<Bytes>, Result<Response<Body>, Infallible>> {
    let maybe_channel = match get_channel(&maybe_query) {
        Ok(res) => res.or(maybe_channel),
        Err(err) => return Err(Ok(new_write_error(err, None, "_channel"))),
    };

    match maybe_channel {
        Some(c) => match parse_and_modify_u8(whole_body, &c) {
            Ok(v) => Ok(Some(Bytes::from(v))),
            Err(err) => return Err(Ok(new_write_error(err, None, "_channel"))),
        },
        None => Ok(None),
    }
}

pub fn get_id(maybe_query: &Option<Query>,
              maybe_path_id: Option<&str>,
              maybe_req: Option<&Bytes>) -> Option<String> {
    match maybe_query {
        Some(q) => match (q.id.as_ref(), maybe_req) {
            (Some(id), Some(req)) => {
                let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
                return Some(j[id].as_str()?.to_string());
            }
            _ => (),
        }
        _ => (),
    }

    match maybe_path_id {
        Some(path_id) => match path_id {
            "_auto" => Some(Uuid::new_v4().to_string()),
            _other => Some(path_id.to_string()),
        }
        None => None,
    }
}

pub fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error>
{
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            Some(channel_id) => {
                let res = rocks::get(&"_channel".to_string(), &channel_id.clone())?;
                let c = Channel::new_vec(res)?;
                return Ok(Some(c));
            }
            None => Ok(None),
        }
    }
}


#[test]
fn test_get_id() {
    let empty_input = None;
    let s = r#"{"my_key":"my_value"}"#;
    let json = Bytes::from(s);


    assert_eq!(get_id(&Some(Query {
        id: Some("my_key".to_string()),
        end: None,
        limit: None,
        until_key: None,
        skip: None,
        direction_reverse: None,
        channel: None,
    }), None, empty_input), None);

    assert_eq!(get_id(&Some(Query {
        id: Some("my_key".to_string()),
        end: None,
        limit: None,
        until_key: None,
        skip: None,
        direction_reverse: None,
        channel: None,
    }), Some("hello"),
                      Some(&json)), Some("my_value".to_string()));

    assert_eq!(get_id(&None, Some("my_key2"), empty_input), Some("my_key2".to_string()));
    assert_eq!(get_id(&None, Some("my_key2"), empty_input), Some("my_key2".to_string()));
    assert!(get_id(&None, Some("_auto"), empty_input).is_some());
    assert_eq!(get_id(&None, None, empty_input), None);
}
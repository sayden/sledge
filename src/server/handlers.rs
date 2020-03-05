use std::convert::Infallible;

use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use uuid::Uuid;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::rocks;
use crate::components::rocks::StreamItem;
use crate::components::storage::Error;
use crate::server::query::Query;
use crate::server::responses::{new_error, new_read_ok, new_write_ok};

pub async fn range(maybe_query: Option<Query>, maybe_path_id: Option<&str>, cf_name: &str, maybe_channel: Option<Channel>) -> Result<Response<Body>, Infallible> {
    let maybe_id = get_id(&maybe_query, maybe_path_id, None);

    let thread_iter = match rocks::range(maybe_query, maybe_id, cf_name, maybe_channel) {
        Ok(iter) => iter,
        Err(err) => return Ok(new_error(err, Some(cf_name))),
    };

    let stream: Box<dyn Stream<Item=StreamItem> + Send + Sync> = box futures::stream::iter(thread_iter);

    match http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream)) {
        Ok(response) => Ok(response),
        Err(err) => Ok(new_error(err, Some(cf_name)))
    }
}

pub async fn put(cf: &str, maybe_query: Option<Query>, maybe_path_id: Option<&str>, req: Body, maybe_channel: Option<Channel>)
                 -> Result<Response<Body>, Response<Body>>
{
    let whole_body = hyper::body::to_bytes(req).await.map_err(|err| new_error(err, Some(cf)))?;
    let id = get_id(&maybe_query, maybe_path_id, Some(&whole_body)).ok_or(new_error("no id found", Some(cf)))?;
    let maybe_data = pass_through_channel(maybe_query, whole_body.as_ref(), maybe_channel)?;
    let data = maybe_data.unwrap_or(whole_body);

    match rocks::put(cf, &id, data) {
        Ok(()) => Ok(new_write_ok(&id, cf)),
        Err(err) => Ok(new_error(err, cf.into())),
    }
}

pub fn get(maybe_query: Option<Query>, cf: &str, id: &str, maybe_channel: Option<Channel>) -> Result<Response<Body>, Response<Body>> {
    let value = match rocks::get(&cf, &id) {
        Ok(res) => res,
        Err(err) => return Ok(new_error(err, Some(&cf)))
    };

    let data = pass_through_channel(maybe_query, value.as_ref(), maybe_channel)?;

    Ok(new_read_ok(data.unwrap_or_else(|| Bytes::from(value)).as_ref(), id, cf))
}

fn pass_through_channel(maybe_query: Option<Query>, whole_body: &[u8], maybe_channel: Option<Channel>) -> Result<Option<Bytes>, Response<Body>> {
    let maybe_channel = match get_channel(&maybe_query) {
        Ok(res) => res.or(maybe_channel),
        Err(err) => return Err(new_error(err, Some("_channel"))),
    };

    match maybe_channel {
        Some(c) => match parse_and_modify_u8(whole_body, &c) {
            Ok(v) => Ok(Some(Bytes::from(v))),
            Err(err) => Err(new_error(err, Some("_channel"))),
        },
        None => Ok(None),
    }
}

pub fn get_id(maybe_query: &Option<Query>,
              maybe_path_id: Option<&str>,
              maybe_req: Option<&Bytes>) -> Option<String> {
    if let Some(q) = maybe_query {
        if let (Some(id), Some(req)) = (q.id_path.as_ref(), maybe_req) {
            let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
            let val: &Value = json_nested_value(id, &j);
            return Some(val.as_str()?.to_string());
        }
    }

    match maybe_path_id {
        Some(path_id) => match path_id {
            "_auto" => Some(Uuid::new_v4().to_string()),
            _ => Some(path_id.to_string()),
        }
        None => None,
    }
}

fn json_nested_value<'a>(k: &str, v: &'a Value) -> &'a Value {
    k.split('.').fold(v, move |acc, x| {
        &acc[x]
    })
}

pub fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error>
{
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            None => Ok(None),
            Some(channel_id) => {
                let res = rocks::get("_channel", &channel_id)?;
                let c = Channel::new_vec(res)?;
                Ok(Some(c))
            }
        }
    }
}


#[test]
fn test_get_id() {
    let empty_input = None;
    let s = r#"{"my_key":"my_value"}"#;
    let json = Bytes::from(s);


    assert_eq!(get_id(&Some(Query {
        id_path: Some("my_key".to_string()),
        end: None,
        limit: None,
        until_key: None,
        skip: None,
        direction_reverse: None,
        channel: None,
    }), None, empty_input), None);

    assert_eq!(get_id(&Some(Query {
        id_path: Some("my_key".to_string()),
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
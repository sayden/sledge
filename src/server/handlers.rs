use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use uuid::Uuid;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::rocks;
use crate::components::rocks::StreamItem;
use crate::components::errors::Error;
use crate::server::query::Query;
use crate::server::responses::{new_read_ok, ResultEmbeddedReply, WriteReply};

pub async fn range(maybe_query: Option<Query>, maybe_path_id: Option<&str>, cf_name: &str, maybe_channel: Option<Channel>) -> Result<Response<Body>, Error> {
    let maybe_id = get_id(&maybe_query, maybe_path_id, None);

    let thread_iter = rocks::range(maybe_query, maybe_id, cf_name, maybe_channel)?;

    let stream: Box<dyn Stream<Item=StreamItem> + Send + Sync> = box futures::stream::iter(thread_iter);

    http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream))
        .map_err(Error::GeneratingResponse)
}

pub async fn put<'a>(cf: &str, maybe_query: &'a Option<Query>, maybe_path_id: Option<&str>, req: Body, maybe_channel: &'a Option<Channel>)
                     -> Result<Response<Body>, Error>
{
    let value = hyper::body::to_bytes(req).await.map_err(Error::BodyParsingError)?;
    let id = get_id(&maybe_query, maybe_path_id, Some(&value)).ok_or(Error::WrongQuery)?;

    let data = match maybe_channel {
        Some(ch) => parse_and_modify_u8(value.as_ref(), ch).map(Bytes::from)?,
        None => value,
    };

    rocks::put(cf, &id, data)?;

    Ok(WriteReply::<&str> {
        result: ResultEmbeddedReply::ok(),
        id: Some(id.as_str()),
    }.into())
}

pub fn get<'a>(cf: &'a str, id: &'a str, maybe_channel: &'a Option<Channel>) -> Result<Response<Body>, Error> {
    let value = rocks::get(&cf, &id)?;

    let data = match maybe_channel {
        Some(ch) => parse_and_modify_u8(value.as_ref(), &ch).map(Bytes::from)?,
        None => Bytes::from(value)
    };

    new_read_ok(data.as_ref(), id)
}

fn get_id(maybe_query: &Option<Query>,
              maybe_path_id: Option<&str>,
              maybe_req: Option<&Bytes>)
              -> Option<String>
{
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
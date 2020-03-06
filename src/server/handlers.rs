use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use uuid::Uuid;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::errors::Error;
use crate::components::rocks;
use crate::server::query::Query;
use crate::server::responses::{new_read_ok, range_result_to_string, ResultEmbeddedReply, WriteReply};

type BytesResult = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;
type BytesResultStream = Box<dyn Stream<Item=BytesResult> + Send + Sync>;
type BytesResultIterator = dyn Iterator<Item=BytesResult> + Send + Sync;

struct IndexedValue {
    key: String,
    value: String,
}

pub async fn create(query: Option<Query>, cf_name: &str) -> Result<Response<Body>, Error>
{
    let id = query.map(|x| x.id_path).flatten().ok_or(Error::MissingID)?;

    let iter = rocks::range(query, None, cf_name, None)?;

    let new_cf_name = format!("{}_by_{}", cf_name, id);

    let res = iter
        .flat_map(|x| json_nested_value(&id, &x.data).as_str()
            .map(|k| IndexedValue { key: format!("{}{}", k, x.id), value: x.id }))
        .map(|x| rocks::put(&new_cf_name, &x.key, Bytes::from(x.value)));

    return Err(Error::MissingID);
}

pub async fn range(query: Option<Query>,
                   path_id: Option<&str>,
                   cf_name: &str,
                   channel: Option<Channel>)
                   -> Result<Response<Body>, Error>
{
    let maybe_id = get_id(&query, path_id, None);

    let thread_iter: Box<BytesResultIterator> = box rocks::range(query, maybe_id, cf_name, channel)?
        .map(|rr| range_result_to_string(&rr))
        .flat_map(|s| s.map(|s| Ok(Bytes::from(s))));

    let stream: BytesResultStream = box futures::stream::iter(thread_iter);

    http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream))
        .map_err(Error::GeneratingResponse)
}

pub async fn put<'a>(cf: &str, query: &'a Option<Query>, path_id: Option<&str>, req: Body, channel: &'a Option<Channel>)
                     -> Result<Response<Body>, Error>
{
    let value = hyper::body::to_bytes(req).await.map_err(Error::BodyParsingError)?;
    let id = get_id(&query, path_id, Some(&value)).ok_or(Error::WrongQuery)?;

    let data = match channel {
        Some(ch) => parse_and_modify_u8(value.as_ref(), ch).map(Bytes::from)?,
        None => value,
    };

    rocks::put(cf, &id, data)?;

    Ok(WriteReply::<&str> {
        result: ResultEmbeddedReply::ok(),
        id: Some(id.as_str()),
    }.into())
}

pub fn get<'a>(cf: &'a str, id: &'a str, channel: &'a Option<Channel>) -> Result<Response<Body>, Error> {
    let value = rocks::get(&cf, &id)?;

    let data = match channel {
        Some(ch) => parse_and_modify_u8(value.as_ref(), &ch).map(Bytes::from)?,
        None => Bytes::from(value)
    };

    new_read_ok(data.as_ref(), id)
}

fn get_id(query: &Option<Query>,
          path_id: Option<&str>,
          req: Option<&Bytes>)
          -> Option<String>
{
    if let Some(q) = query {
        if let (Some(id), Some(req)) = (q.id_path.as_ref(), req) {
            let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
            let val: &Value = json_nested_value(id, &j);
            return Some(val.as_str()?.to_string());
        }
    }

    match path_id {
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
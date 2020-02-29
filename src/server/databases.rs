use std::convert::Infallible;
use warp::Reply;
use std::sync::Arc;
use crate::components::storage::{Storage, Error};
use bytes::Bytes;
use uuid::Uuid;

use serde_json::Value;
use crate::server::{errors, responses};
use crate::server::requests::Query;
use crate::components::kv::KV;
use std::iter::FromIterator;
use warp::reply::Response;
use hyper::Body;
use crate::channels::parser::{Channel, parse_and_modify_u8};


pub async fn handle_get(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                        keyspace: String,
                        id: String,
                        req: Query)
                        -> Result<impl Reply, Infallible>
{
    let v1 = db.lock().await;

    let maybe_channel = match get_channel(&v1, req.channel) {
        Ok(res) => res,
        Err(err) => return errors::new_boxed_read(err.to_string(), Some("_channel".to_string())),
    };

    let value: String = match v1.get(Some(keyspace.clone()), id) {
        Err(err) => return errors::new_boxed_read(err.to_string(), Some(keyspace)),
        Ok(v) => v,
    };

    let data = match maybe_channel {
        Some(c) => match parse_and_modify_u8(value.as_ref(), &c) {
            Ok(v) => Bytes::from(v),
            Err(err) => return errors::new_boxed_read(err.to_string(), Some("_channel".to_string())),
        },
        None => Bytes::from(value),
    };

    let maybe_value = serde_json::from_slice::<Value>(data.as_ref());
    match maybe_value {
        Ok(value) => responses::new_boxed_read(Some(Box::new([value])), Some(keyspace)),
        Err(err) => errors::new_boxed_read(Error::Serialization(err).to_string(), Some(keyspace)),
    }
}

pub async fn handle_range(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                          keyspace: String,
                          id: String,
                          query: Query)
                          -> Result<Box<dyn Reply>, Infallible> {
    let v1 = db.lock().await;

    let maybe_channel = match get_channel(&v1, query.clone().channel) {
        Ok(res) => res,
        Err(err) => return errors::new_boxed_read(err.to_string(), Some("_channel".to_string())),
    };

    let mut full_query = query;
    full_query.id = Some(id);

    let iter: Box<dyn Iterator<Item=KV>> = match v1.range(Some(keyspace.clone()), full_query) {
        Err(err) => return errors::new_boxed_read(err.to_string(), Some(keyspace)),
        Ok(x) => x,
    };

    if maybe_channel.is_some() {
        let ch = maybe_channel.unwrap();
        let ch_name = ch.name.clone();
        let iter2 = iter
            .map(|x| {
                match parse_and_modify_u8(x.value.as_ref(), &ch) {
                    Ok(v) => Vec::from(v),
                    Err(err) => {
                        log::warn!("error trying to pass value through channel '{}': {}", ch_name, err.to_string());
                        x.value
                    }
                }
            })
            .flat_map(move |x| {
                let mut v = x;

                v.push('\n' as u8);
                v.into_iter()
            });

        let byt = Bytes::from_iter(iter2);

        Ok(Box::new(into_response(byt)))
    } else {
        let iter3 = iter.flat_map(move |x| {
            let mut v = x.value;

            v.push('\n' as u8);
            v.into_iter()
        });

        let byt = Bytes::from_iter(iter3);

        Ok(Box::new(into_response(byt)))
    }
}

pub async fn handle_put(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                        maybe_path_id: Option<String>,
                        maybe_query: Option<Query>,
                        keyspace: String,
                        req: Bytes)
                        -> Result<impl Reply, Infallible>
{
    let id = match get_id(maybe_query.clone(), maybe_path_id, &req) {
        None => return errors::new_write("no id found for your document", Some(keyspace)),
        Some(s) => s,
    };

    let v1_locked = db.lock().await;
    let mut v1 = v1_locked;

    let maybe_channel = match get_channel(&v1, maybe_query.and_then(|x| x.channel)) {
        Ok(res) => res,
        Err(err) => return errors::new_write_string(err.to_string(), Some("_channel".to_string())),
    };

    let data = match maybe_channel {
        Some(c) => match parse_and_modify_u8(req.as_ref(), &c) {
            Ok(v) => Bytes::from(v),
            Err(err) => return errors::new_write(err.to_string().as_ref(), Some("_channel".to_string())),
        },
        None => req,
    };

    match v1.put(Some(keyspace.clone()), id.clone(), data) {
        Ok(_) => responses::new_write(Some(id), Some(keyspace)),
        Err(err) => errors::new_write(&err.to_string(), Some(keyspace)),
    }
}

pub async fn handle_start(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                          keyspace: String,
                          req: Query)
                          -> Result<Box<dyn Reply>, Infallible> {
    let v1 = db.lock().await;

    let iter: Box<dyn Iterator<Item=KV>> = match v1.start(Some(keyspace.clone())) {
        Err(err) => return errors::new_boxed_read(err.to_string(), Some(keyspace)),
        Ok(x) => x,
    };

    let iter2 = iter
        .flat_map(move |x| {
            let mut v = x.value;
            v.push('\n' as u8);
            v.into_iter()
        });

    let byt = Bytes::from_iter(iter2);

    Ok(Box::new(into_response(byt)))
}

fn into_response(byt: Bytes) -> Response {
    ::http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(byt))
        .unwrap()
}

fn get_id(maybe_query: Option<Query>,
          maybe_path_id: Option<String>,
          req: &Bytes) -> Option<String> {
    if (&maybe_query).is_some() && maybe_query.clone().unwrap().id.is_some() {
        let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
        return Some(j[maybe_query?.id?].as_str()?.to_string());
    }

    if maybe_path_id.is_some() {
        match maybe_path_id.clone()?.as_str() {
            "_auto" => Some(Uuid::new_v4().to_string()), // generate key
            _ => maybe_path_id,
        }
    } else {
        return None; //No ?id= nor /db/{db}/{id} nor /db/{db}/auto so no way to know the ID of this
    }
}

fn get_channel(db: &tokio::sync::MutexGuard<'_, Box<dyn Storage + Send + Sync>>,
               maybe_channel: Option<String>)
               -> Result<Option<Channel>, Error>
{
    match maybe_channel {
        Some(channel_id) => {
            let channel_json = db.get(Some("_channel".to_string()), channel_id.clone())?;

            let c = Channel::new(channel_json.as_str())?;

            Ok(Some(c))
        }
        None => Ok(None),
    }
}


#[test]
fn test_get_key() {
    let byt = Bytes::new();
    let s = r#"{"my_key":"my_value"}"#;
    let bytes_with_content = Bytes::from(s);
    assert_eq!(get_id(Some(InsertQueryReq { id: Some("my_key".to_string()), channel: None }), None, &byt), None);
    assert_eq!(get_id(Some(InsertQueryReq { id: Some("my_key".to_string()), channel: None }), Some("hello".to_string()),
                      &bytes_with_content), Some("my_value".to_string()));
    assert_eq!(get_id(None, Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));
    assert_eq!(get_id(None, Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));
    assert!(get_id(None, Some("_auto".to_string()), &byt).is_some());
    assert_eq!(get_id(None, None, &byt), None);
}
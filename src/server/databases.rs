use std::convert::Infallible;
use warp::Reply;
use std::sync::Arc;
use crate::components::storage::{Storage, Error};
use bytes::Bytes;
use uuid::Uuid;

use serde_json::Value;
use crate::server::{errors, responses};
use crate::server::requests::{InsertQueryReq, Query};
use crate::components::kv::KV;
use std::iter::FromIterator;
use warp::reply::Response;
use hyper::Body;
use crate::components::storage;


pub async fn get(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, key: String)
                 -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;

    let data: String = match v1.get(Some(keyspace.clone()), key) {
        Err(err) => return errors::new_read(err.to_string(), Some(keyspace)),
        Ok(v) => v,
    };

    let maybe_value = serde_json::from_str::<Value>(data.as_str());
    match maybe_value {
        Ok(value) => responses::new_read(Some(Box::new([value])), Some(keyspace)),
        Err(err) => errors::new_read(Error::Serialization(err).to_string(), Some(keyspace)),
    }
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

pub async fn start(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                   keyspace: String)
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

pub async fn range(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                   keyspace: String,
                   id: String,
                   query: Query)
                   -> Result<Box<dyn Reply>, Infallible> {
    let v1 = db.lock().await;

    let mut full_query = query;
    full_query.id = Some(id);

    let iter: Box<dyn Iterator<Item=KV>> = match v1.range(Some(keyspace.clone()), full_query) {
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

pub async fn since(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                   keyspace: String)
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

pub async fn handler_put(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                         maybe_path_id: Option<String>,
                         maybe_query: Option<InsertQueryReq>,
                         keyspace: String,
                         req: Bytes)
                         -> Result<impl Reply, Infallible>
{
    let id = match get_id(maybe_query, maybe_path_id, &req) {
        None => return errors::new_write("no id found for your document", Some(keyspace)),
        Some(s) => s,
    };

    let v1_locked = db.lock().await;
    let mut v1 = v1_locked;

    match v1.put(Some(keyspace.clone()), id.clone(), req) {
        Ok(_) => responses::new_write(Some(id), Some(keyspace)),
        Err(err) => errors::new_write(&err.to_string(), Some(keyspace)),
    }
}

fn get_id(maybe_query: Option<InsertQueryReq>, maybe_path_id: Option<String>, req: &Bytes) -> Option<String> {
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
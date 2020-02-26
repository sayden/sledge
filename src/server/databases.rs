use std::convert::Infallible;
use warp::Reply;
use std::sync::Arc;
use crate::components::storage::{Storage, Error};
use bytes::Bytes;
use uuid::Uuid;

use serde_json::Value;
use crate::server::{errors, responses};
use crate::server::requests::{InsertQueryReq, GetReq};


pub async fn get(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, key: String)
                 -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;

    let res: Result<String, Error> = v1.get(Some(keyspace.clone()), key);
    if res.is_err() {
        return errors::new_read(res.unwrap_err().to_string(), Some(keyspace));
    }

    let data = res.unwrap();
    let maybe_value = serde_json::from_str::<Value>(data.as_str());
    match maybe_value {
        Ok(value) => responses::new_read(Some(Box::new([value])), Some(keyspace)),
        Err(err) => errors::new_read(Error::Serialization(err).to_string(), Some(keyspace)),
    }
}


pub async fn query(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, req: GetReq)
                   -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;

    let res: Result<String, Error> = v1.get(Some(keyspace.clone()), req.key);
    if res.is_err() {
        return errors::new_read(res.unwrap_err().to_string(), Some(keyspace));
    }

    let data = res.unwrap();
    match serde_json::from_str::<Value>(data.as_str()) {
        Ok(value) => responses::new_read(Some(Box::new([value])), Some(keyspace)),
        Err(err) => errors::new_read(Error::Serialization(err).to_string(), Some(keyspace)),
    }
}

pub async fn handler_put(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
                         maybe_path_id: Option<String>,
                         maybe_query: Option<InsertQueryReq>,
                         keyspace: String,
                         req: Bytes)
                         -> Result<impl Reply, Infallible>
{
    let id = get_id(maybe_query, maybe_path_id, &req);
    if id.is_none() {
        return errors::new_write("no id found for your document", Some(keyspace));
    }

    let v1_locked = db.lock().await;
    let mut v1 = v1_locked;

    match v1.put(Some(keyspace.clone()), id.clone().unwrap(), req) {
        Ok(_) => responses::new_write(id, Some(keyspace)),
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
    assert_eq!(get_id(Some(InsertQueryReq{ id: Some("my_key".to_string()), channel: None }), None, &byt), None);
    assert_eq!(get_id(Some(InsertQueryReq{ id: Some("my_key".to_string()), channel: None }), Some("hello".to_string()),
                      &bytes_with_content), Some("my_value".to_string()));
    assert_eq!(get_id(None, Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));
    assert_eq!(get_id(None, Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));
    assert!(get_id(None, Some("_auto".to_string()), &byt).is_some());
    assert_eq!(get_id(None, None, &byt), None);
}
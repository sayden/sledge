use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Reply;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::server::filters::{GetReq, InsertQueryReq};
use crate::components::storage::{Storage, Error};
use bytes::{Bytes, Buf};
use uuid::Uuid;
use warp::reply::Response;
use hyper::Body;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct StandardReply {
    error: bool,
    cause: Option<String>,
    data: Option<Box<[Value]>>,
}

fn new_error_reply(cause: String) -> Result<StandardReply, Infallible> {
    Ok(StandardReply {
        error: true,
        cause: Some(cause),
        data: None,
    })
}

fn new_data_reply(data: Option<Box<[Value]>>) -> Result<StandardReply, Infallible> {
    Ok(StandardReply {
        error: false,
        cause: None,
        data,
    })
}

impl warp::Reply for StandardReply {
    fn into_response(self) -> Response {
        Response::new(serde_json::to_string(&self).unwrap().into())
    }
}

pub async fn stats(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>) -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    Ok(warp::reply::json(&v1.stats()))
}

pub async fn ok() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}

pub async fn get(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, key: String)
                 -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;

    let res: Result<String, Error> = v1.get(Some(keyspace), key);
    if res.is_err() {
        return new_error_reply(res.unwrap_err().to_string());
    }

    let data = res.unwrap();
    let maybe_value = serde_json::from_str::<Value>(data.as_str());
    match maybe_value {
        Ok(value) => new_data_reply(Some(Box::new([value]))),
        Err(err) => new_error_reply(Error::Serialization(err).to_string()),
    }
}


pub async fn query(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, req: GetReq)
                   -> Result<impl Reply, Infallible> {
    log::debug!("query: {}", req);
    let v1_locked = db.lock().await;
    let v1 = v1_locked;

    let res: Result<String, Error> = v1.get(Some(keyspace), req.key);
    if res.is_err() {
        return new_error_reply(res.unwrap_err().to_string());
    }

    let data = res.unwrap();
    match serde_json::from_str::<Value>(data.as_str()) {
        Ok(value) => new_data_reply(Some(Box::new([value]))),
        Err(err) => new_error_reply(Error::Serialization(err).to_string()),
    }
}

pub async fn insert(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>,
                    keyspace: String,
                    key: Option<String>,
                    query: Option<InsertQueryReq>,
                    req: Bytes)
                    -> Result<impl Reply, Infallible>
{
    let value = match std::str::from_utf8(req.bytes()) {
        Ok(val) => val,
        Err(err) => return new_error_reply(format!("error parsing id value from body: {}", err.to_string()))
    };

    let id = get_key(query, key, &req);
    if id.is_none() {
        return new_error_reply("no id found for your document".to_string());
    }

    let v1_locked = db.lock().await;
    let mut v1 = v1_locked;

    match v1.put(Some(keyspace), id.clone().unwrap(), value.to_string()) {
        Ok(_) => new_data_reply(None),
        Err(err) => new_error_reply(err.to_string()),
    }
}

fn get_key(query: Option<InsertQueryReq>, key_in_path: Option<String>, req: &Bytes) -> Option<String> {
    if query.clone().is_some() & &query.clone()?.id.is_some() {
        let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
        return Some(j[query?.id?].as_str()?.to_string());
    }

    if key_in_path.is_some() {
        match key_in_path.clone()?.as_str() {
            "auto" => Some(Uuid::new_v4().to_string()), // generate key
            other => key_in_path,
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
    assert_eq!(get_key(Some(InsertQueryReq { id: Some("my_key".to_string()) }), None, &byt), None);
    assert_eq!(get_key(Some(InsertQueryReq {
        id: Some("my_key".to_string())
    }), Some("hello".to_string()), &bytes_with_content), Some("my_value".to_string()));

    assert_eq!(get_key(Some(InsertQueryReq {
        id: None
    }), Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));

    assert_eq!(get_key(Some(InsertQueryReq {
        id: None
    }), Some("my_key2".to_string()), &byt), Some("my_key2".to_string()));

    assert!(get_key(Some(InsertQueryReq { id: None }), Some("auto".to_string()), &byt).is_some());
    assert_eq!(get_key(Some(InsertQueryReq { id: None }), None, &byt), None);
}
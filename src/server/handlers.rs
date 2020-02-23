use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Reply;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::server::filters::{InsertReq, GetReq};
use crate::components::storage::Storage;

pub async fn stats(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>) -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    Ok(warp::reply::json(&v1.stats()))
}

pub async fn ok() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}

pub async fn get(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, req: GetReq)
                 -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    match v1.get(Some(keyspace), req.key) {
        Ok(res) => match res {
            Some(res) => Ok(warp::reply::with_status(res, StatusCode::OK)),
            None => {
                println!("Not found?");
                Ok(warp::reply::with_status(r#"{"error":true}"#.to_string(), StatusCode::OK))
            }
        },
        Err(err) => {
            println!("{}", err);
            Ok(warp::reply::with_status(r#"{"error":true}"#.to_string(), StatusCode::OK))
        }
    }
}

pub async fn insert(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>, keyspace: String, req: InsertReq)
                    -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let mut v1 = v1_locked;
    match v1.put(Some(keyspace), req.key, req.value) {
        Ok(_) => Ok(warp::reply::with_status("Ok!", StatusCode::OK)),
        Err(err) => {
            println!("{}", err);
            Ok(warp::reply::with_status(r#"{"error":true}"#, StatusCode::OK))
        }
    }
}
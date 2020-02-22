use std::convert::Infallible;
use warp::http::StatusCode;
use crate::components::api::V1;
use warp::Reply;
use std::sync::{Arc};
use tokio::sync::Mutex;

pub async fn stats(db: Arc<Mutex<V1>>) -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    Ok(warp::reply::json(&v1.stats()))
}
pub async fn ok() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}
pub async fn insert(db: Arc<Mutex<V1>>) -> Result<impl Reply, Infallible> {
    // Just return a JSON array of todos, applying the limit and offset.
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}
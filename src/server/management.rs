use std::convert::Infallible;
use crate::components::storage::{Storage};
use warp::Reply;
use std::sync::Arc;
use warp::http::StatusCode;

pub async fn stats(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>) -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    Ok(warp::reply::json(&v1.stats()))
}

pub async fn ok() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}
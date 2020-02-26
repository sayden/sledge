use std::convert::Infallible;
use crate::components::storage::Storage;
use warp::{Reply, Filter, Rejection};
use std::sync::Arc;
use warp::http::StatusCode;
use crate::server::filters::with_db;

/**
 * Stats / Health
*/

/// GET /healthz
pub(crate) fn healthz() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and_then(|| ok())
}

/// GET /stats
pub(crate) fn status(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                     -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(|db| stats(db))
}

pub async fn stats(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>) -> Result<impl Reply, Infallible> {
    let v1_locked = db.lock().await;
    let v1 = v1_locked;
    Ok(warp::reply::json(&v1.stats()))
}

pub async fn ok() -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status("Ok!", StatusCode::OK))
}
use super::handlers;
use warp::{Filter, Rejection, Reply};
use crate::components::api::V1;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;
use warp::http::StatusCode;

/// Filters combined.
pub fn all(db: Arc<Mutex<V1>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    healthz(db.clone())
        .or(status(db.clone()))
        .or(insert(db.clone()))
}

/// GET /healthz
pub fn healthz(db: Arc<Mutex<V1>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and_then(|| handlers::ok())
}

/// GET /stats
pub fn status(db: Arc<Mutex<V1>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(|x| handlers::stats(x))
}

/// POST /db with JSON body
pub fn insert(db: Arc<Mutex<V1>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("insert")
        .and(warp::post())
        .and(warp::body::json())
//        .and(with_db(db))
        .and_then(|db| handlers::insert(db))
}


fn with_db(db: Arc<Mutex<V1>>) -> impl Filter<Extract=(Arc<Mutex<V1>>, ), Error=Infallible> + Clone {
    warp::any().map(move || db.clone())
}
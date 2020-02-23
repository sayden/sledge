use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;
use serde::{Serialize, Deserialize};
use crate::components::storage::Storage;
use crate::server::handlers;


#[derive(Serialize, Deserialize)]
pub struct InsertReq {
    pub(crate) key: String,
    pub(crate) value: String,
}

/// Filters combined.
pub fn all(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    healthz()
        .or(status(db.clone()))
        .or(insert(db.clone()))
        .or(get(db.clone()))
}

/// GET /healthz
pub fn healthz() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and_then(|| handlers::ok())
}

#[derive(Deserialize, Serialize)]
pub struct GetReq {
    pub(crate) key: String
}

/// GET /key
pub fn get(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("/" / String)
        .and(warp::query::<GetReq>())
        .and(warp::get())
        .and(with_db(db))
        .and_then(|keyspace, doc, db| handlers::get(db, keyspace, doc))
}

/// GET /stats
pub fn status(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
              -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(|x| handlers::stats(x))
}

/// POST /db with JSON body
pub fn insert(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
              -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("/" / String)
        .and(warp::post())
        .and(with_db(db))
        .and(warp::body::json())
        .and_then(|keyspace: String, db, doc: InsertReq| handlers::insert(db, keyspace, doc))
}


fn with_db(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=(Arc<Mutex<Box<dyn Storage + Send + Sync>>>, ), Error=Infallible> + Clone {
    warp::any().map(move || db.clone())
}
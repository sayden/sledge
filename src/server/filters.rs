use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::convert::Infallible;
use serde::{Serialize, Deserialize};
use crate::components::storage::Storage;
use crate::server::handlers;
use std::fmt::Display;
use serde::export::Formatter;


#[derive(Serialize, Deserialize, Clone)]
pub struct InsertQueryReq {
    pub(crate) id: Option<String>,
}

impl Display for InsertQueryReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "key: {}", self.id.as_ref().unwrap_or(&"not found".to_string()))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetReq {
    pub(crate) key: String
}

impl Display for GetReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "key: {}", self.key)
    }
}

/// Filters combined.
pub fn all(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    healthz()
        .or(status(db.clone()))
        .or(insert(db.clone()))
        .or(insert_id_in_json(db.clone()))
        .or(get(db.clone()))
        .or(query(db.clone()))
}

/// GET /healthz
pub fn healthz() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and_then(|| handlers::ok())
}

/// GET /db/{db}/{key}
pub fn get(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(|keyspace, key: String, db| handlers::get(db, keyspace, key))
}

/// POST /db/{db}
pub fn query(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String)
        .and(warp::body::json())
        .and(warp::post())
        .and(with_db(db))
        .and_then(|keyspace, doc: GetReq, db| handlers::query(db, keyspace, doc))
}

/// GET /stats
pub fn status(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
              -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(|x| handlers::stats(x))
}

/// PUT /db with JSON body
pub fn insert(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
              -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::body::bytes())
        .and_then(|keyspace: String, key: String, db, doc| handlers::insert(db, keyspace, match key.is_empty() {
            true => None,
            false => Some(key),
        }, None, doc))
}

/// PUT /db with JSON body
pub fn insert_id_in_json(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
                         -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::body::bytes())
        .and(warp::query::<InsertQueryReq>())
        .and_then(|keyspace: String, db, doc, query| handlers::insert(db, keyspace, None, Some(query), doc))
}


fn with_db(db: Arc<Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=(Arc<Mutex<Box<dyn Storage + Send + Sync>>>, ), Error=Infallible> + Clone {
    warp::any().map(move || db.clone())
}
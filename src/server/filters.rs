use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use std::convert::Infallible;
use serde::{Serialize, Deserialize};
use crate::components::storage::Storage;
use crate::server::{handlers, management};
use std::fmt::Display;
use serde::export::Formatter;
use crate::server::handlers::handler_put;
use crate::server::channels::insert_channel;


#[derive(Serialize, Deserialize, Clone)]
pub struct InsertQueryReq {
    pub(crate) id: Option<String>,
    pub(crate) channel: Option<String>,
}

impl Display for InsertQueryReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or(r#"{"error":true}"#.to_string()))
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
pub fn all(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    healthz()
        .or(status(db.clone()))
        .or(insert_doc(db.clone()))
        .or(insert_doc_id_in_json(db.clone()))
        .or(get(db.clone()))
        .or(query(db.clone()))
        .or(insert_channel(db.clone()))
}

/**
 * Stats / Health
*/

/// GET /healthz
pub fn healthz() -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("healthz")
        .and(warp::get())
        .and_then(|| management::ok())
}

/// GET /stats
pub fn status(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
              -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("stats")
        .and(warp::get())
        .and(with_db(db))
        .and_then(|x| management::stats(x))
}

/**
 * Read operations
*/

/// GET /db/{db}/{key}
pub fn get(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::get())
        .and(with_db(db))
        .and_then(|keyspace, key: String, db| handlers::get(db, keyspace, key))
}

/// POST /db/{db}
pub fn query(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String)
        .and(warp::body::json())
        .and(warp::post())
        .and(with_db(db))
        .and_then(|keyspace, doc: GetReq, db| handlers::query(db, keyspace, doc))
}

/**
 * Write operations
*/

/**
 * `PUT /db/{db}/{id}?channel={id}`
 * `PUT /db/{db}/_auto?channel={id}`
*/
pub fn insert_doc(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::query::<InsertQueryReq>())
        .and(warp::body::bytes())
        .and_then(|keyspace, path_id: String, db, query, body| handler_put(db, path_id.into(), Some(query), keyspace, body))
}

/**
 * `PUT /db/{db}?id={json_field}&channel={id}`
*/
pub fn insert_doc_id_in_json(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::query::<InsertQueryReq>())
        .and(warp::body::bytes())
        .and_then(|keyspace, db, query, body| handler_put(db, None, Some(query), keyspace, body))
}

pub fn with_db(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
               -> impl Filter<Extract=(Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>, ), Error=Infallible> + Clone {
    warp::any().map(move || db.clone())
}
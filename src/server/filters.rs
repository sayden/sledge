use warp::{Filter, Rejection, Reply};
use std::sync::Arc;
use std::convert::Infallible;

use crate::components::storage::Storage;
use crate::server::databases;

use crate::server::databases::handle_put;
use crate::server::management::{healthz, status};
use crate::server::requests::Query;

/// Filters combined.
pub fn all(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    healthz()
        .or(status(db.clone()))
        .or(insert_doc(db.clone()))
        .or(insert_doc_id_in_json(db.clone()))
        .or(start(db.clone()))
        .or(range(db.clone()))
        .or(get(db.clone()))
        .or(insert_channel(db.clone()))
}

/// GET /db/{db}/{key}
/// Returns the value from the specified key in the specified database
///
/// # Examples
///
/// ```
/// curl -XGET localhost:3030/db/other_db/00001
/// ```
///
/// Returns:
/// ```
/// {
///   "result": {
///     "error": false,
///     "cause": null,
///     "db": "other_db"
///   },
///   "data": [
///     {"age":35}
///   ]
/// }
/// ```
pub fn get(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
           -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::query::<Query>())
        .and_then(|keyspace, key: String, db, req| databases::handle_get(db, keyspace, key, req))
}

/// GET /db/{db}/_all?limit={usize}&skip={usize}&until_key={String}&end={String}&direction_forward={bool}?channel={String}
pub fn start(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone
{
    warp::path!("db" / String / "_all")
        .and(warp::get())
        .and(with_db(db))
        .and(warp::query::<Query>())
        .and_then(|keyspace, db, req| databases::handle_start(db, keyspace, req))
}

/// GET /db/{db}/_range/{id}?limit={usize}&skip={usize}&until_key={String}&end={String}&direction_forward={bool}?channel={String}
pub fn range(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone
{
    warp::path!("db" / String / "_range" / String)
        .and(warp::get())
        .and(with_db(db))
        .and(warp::query::<Query>())
        .and_then(|keyspace, id, db, query| databases::handle_range(db, keyspace, id, query))
}

///
/// `PUT /db/{db}/{id}?channel={id}`
/// `PUT /db/{db}/_auto?channel={id}`
///
pub fn insert_doc(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                  -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::query::<Query>())
        .and(warp::body::bytes())
        .and_then(|keyspace, path_id: String, db, query, body| {
            handle_put(db, path_id.into(), Some(query), keyspace, body)
        })
}

/**
 * `PUT /db/{db}?id={json_field}&channel={id}`
*/
pub fn insert_doc_id_in_json(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                             -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("db" / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::query::<Query>())
        .and(warp::body::bytes())
        .and_then(|keyspace, db, query, body| handle_put(db, None, Some(query), keyspace, body))
}

pub fn with_db(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
               -> impl Filter<Extract=(Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>, ), Error=Infallible> + Clone {
    warp::any().map(move || db.clone())
}

///
/// `PUT /channel/{id}`
///
pub fn insert_channel(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                      -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone
{
    warp::path!("_channel" / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::body::bytes())
        .and_then(|id: String, db, body| handle_put(db, match id.is_empty() {
            true => None,
            _ => Some(id)
        }, None, "_channel".to_string(), body))
}
use std::sync::Arc;
use crate::components::storage::Storage;
use warp::{Filter, Rejection, Reply};
use serde::{Serialize, Deserialize};
use crate::server::filters::{with_db, InsertQueryReq};
use bytes::Bytes;
use std::convert::Infallible;
use crate::server::handlers::handler_put;

/**
 * `PUT /channel/{id}`
*/
pub fn insert_channel(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>)
                      -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    warp::path!("_channel" / String)
        .and(warp::put())
        .and(with_db(db))
        .and(warp::body::bytes())
        .and_then(|id: String, db, body| handler_put(db, match id.is_empty() {
            true => None,
            _ => Some(id)
        }, None, "_channel".to_string(), body))
}

// pub async fn insert_channel_handler(db: Arc<tokio::sync::Mutex<Box<dyn Storage + Send + Sync>>>,
//                          maybe_path_id: Option<String>,
//                          maybe_query: Option<InsertQueryReq>,
//                          keyspace: String,
//                          req: Bytes)
//                          -> Result<impl Reply, Infallible>{
// }
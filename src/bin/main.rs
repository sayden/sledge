use warp::{Filter, Rejection, Reply};
use log::info;
use std::sync::Arc;
use sledge::components::api::V1;
use sledge::components::storage::get_storage;
use std::borrow::Borrow;
use std::fs::FileType;
use sledge::server::filters;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Arc::new(Mutex::new(V1::new(get_storage(env!("STORAGE"), "/tmp/storage"))));
    let api = filters::all(app.clone());

    let routes = api.with(warp::log("sledge"));

    // Start up the server...
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
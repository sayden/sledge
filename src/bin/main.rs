use warp::Filter;
use std::sync::Arc;
use sledge::components::storage::get_storage;
use sledge::server::filters;
use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();

    let maybe_storage = env::var("FEEDB_STORAGE").unwrap();
    let maybe_path = env::var("FEEDB_PATH").unwrap();

    let db = Arc::new(tokio::sync::Mutex::new(get_storage(maybe_storage.as_str(), maybe_path.as_str())));

    let api = filters::all(db);

    let routes = api.with(warp::log("sledge"));

    // Start up the server...
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
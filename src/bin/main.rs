use warp::{Filter};
use std::sync::{Arc};
use sledge::components::storage::get_storage;
use sledge::server::filters;

#[tokio::main]
async fn main() {
    env_logger::init();

//    let app = Arc::new(Mutex::new(V1::new(get_storage(env!("STORAGE"), "/tmp/storage"))));
    let db = Arc::new(tokio::sync::Mutex::new(get_storage(env!("STORAGE"), "/tmp/storage")));
    let api = filters::all(db);

    let routes = api.with(warp::log("sledge"));

    // Start up the server...
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
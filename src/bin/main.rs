use warp::Filter;
use log::info;
use std::sync::{Mutex, Arc};
use sledge::components::api::V1;
use sledge::components::storage::get_storage;
use std::borrow::Borrow;
use std::fs::FileType;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = Arc::new(Mutex::new(V1::new(get_storage(env!("STORAGE"), "/tmp/storage"))));
    let app = warp::any().map(move||app.clone());
    let root = warp::path("/");
    let index = root.and(warp::path::end());

    let iamok = warp::path!("ok")
        .map(|| "Ok!");

    let stats = warp::path!("stats")
        .map(|| {
            let db = app.lock().unwrap();
            db.stats().to_string()
        });

    let insert = warp::path!("db/asdasd")
//        .and(warp::filters::method::post())
        .map(|| {
            let db = app.lock().unwrap();
            db.stats().to_string()
        });

    let routes = warp::get().and(iamok.or(stats).or(insert));

    info!("Starting server...");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
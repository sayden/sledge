use warp::Filter;
use log::info;
use crate::components::storage::get_storage;
use crate::components::api::V1;

#[tokio::main]
async fn main() {
    env_logger::init();

    let st = get_storage(env!("STORAGE"), "/tmp/storage");
    let app = V1 {
        storage: st,
    };

    let iamok = warp::path!("ok").map(|| "Ok!");
    let stats = warp::path!("stats").map(|| app.stats().to_string());

    let routes = warp::get().and(iamok);

    info!("Starting server...");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
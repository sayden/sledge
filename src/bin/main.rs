extern crate hyper;
extern crate tokio;

use std::env;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use futures_util::future;
use hyper::service::Service;
use hyper::Server;

use sledge::components::rocks;
use sledge::server::service::Svc;

pub struct MakeSvc {
    db: Arc<RwLock<rocksdb::DB>>,
}

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future { future::ok(Svc::new(self.db.clone())) }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "127.0.0.1:3000".parse().unwrap();

    let maybe_path = env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string());
    let db = Arc::new(RwLock::new(rocks::new_storage(maybe_path)));

    let server = Server::bind(&addr).serve(MakeSvc { db });

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

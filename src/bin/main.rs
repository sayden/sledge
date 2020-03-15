extern crate hyper;
extern crate tokio;

use std::env;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::service::Service;
use hyper::{Body, Request, Response, Server};

use sledge::channels::channel::Channel;
use sledge::components::errors::Error;
use sledge::components::rocks;
use sledge::server::handlers;
use sledge::server::handlers::{PutRequest, SinceRequest};
use sledge::server::query::Query;

fn get_query(uri: &Uri) -> Option<Query> { serde_urlencoded::from_str::<Query>(uri.query()?).ok() }

fn get_path(p: &str) -> Vec<&str> {
    //TODO use some library for this
    p.split('/').filter(|x| x != &"").collect()
}

struct SPath<'a> {
    route: Option<&'a str>,
    cf: Option<&'a str>,

    id_or_action: Option<&'a str>,
    param1: Option<&'a str>,

    id_or_action2: Option<&'a str>,
    param2: Option<&'a str>,
}

struct CommonRequest<'a> {
    ch: Option<Channel>,
    path: SPath<'a>,
    query: Option<Query>,
    body: Body,
}

struct ReadRequest<'a> {
    req: CommonRequest<'a>,
    db: RwLockReadGuard<'a, rocksdb::DB>,
}

struct WriteRequest<'a> {
    c: CommonRequest<'a>,
    db: RwLockWriteGuard<'a, rocksdb::DB>,
}

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

    fn call(&mut self, _: T) -> Self::Future {
        future::ok(Svc {
            db: self.db.clone(),
        })
    }
}

#[derive(Debug)]
pub struct Svc {
    db: Arc<RwLock<rocksdb::DB>>,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let query = get_query(&parts.uri);
        let path = get_path(parts.uri.path());

        let path = SPath {
            route: path.get(0).cloned(),
            cf: path.get(1).cloned(),
            id_or_action: path.get(2).cloned(),
            param1: path.get(3).cloned(),
            id_or_action2: path.get(4).cloned(),
            param2: path.get(5).cloned(),
        };

        let ch = match self.get_channel(&query) {
            Ok(res) => res,
            Err(err) => return future::ok(err.into()),
        };

        let common = CommonRequest {
            ch,
            path,
            query,
            body,
        };

        let res: Result<Response<Body>, Error> = match parts.method {
            Method::GET => {
                let db = self.db.read().unwrap();
                self.get_handlers(ReadRequest { req: common, db })
            }
            Method::PUT => {
                let db = self.db.write().unwrap();
                self.put_handlers(WriteRequest { c: common, db })
            }
            Method::POST => {
                let db = self.db.read().unwrap();
                self.post_handlers(ReadRequest { req: common, db })
            }
            _ => Err(Error::MethodNotFound),
        };

        match res {
            Ok(res) => future::ok(res),
            Err(err) => future::ok(err.into()),
        }
    }
}

impl Svc {
    fn put_handlers(&self, req: WriteRequest<'_>) -> Result<Response<Body>, Error> {
        match (req.c.path.route, req.c.path.cf, req.c.path.id_or_action) {
            // (Some("_db"), Some(cf), Some("_create_secondary_index"))=>Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
            (Some("_db"), Some(cf), Some("_create_db")) => handlers::create_db(self.db.clone(), cf),
            (Some("_db"), Some(cf), path_id) => {
                handlers::put(PutRequest {
                    cf,
                    query: &req.c.query,
                    path_id,
                    req: req.c.body,
                    ch: req.c.ch,
                    db: req.db,
                })
                .and_then(Ok)
                .or_else(|err| Ok(err.into()))
            }
            _ => Err(Error::WrongQuery),
        }
    }

    fn post_handlers(&self, req: ReadRequest<'_>) -> Result<Response<Body>, Error> {
        match (
            req.req.path.route,
            req.req.path.cf,
            req.req.path.id_or_action,
        ) {
            (Some("_sql"), ..) => handlers::sql(self.db.clone(), req.req.query, req.req.body),
            (Some("_db"), Some(cf_name), Some(id)) => {
                handlers::get(req.db, cf_name, id, req.req.query)
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into()))
            }
            _ => Err(Error::WrongQuery),
        }
    }

    fn get_handlers(&self, r: ReadRequest<'_>) -> Result<Response<Body>, Error> {
        match (
            r.req.path.route,
            r.req.path.cf,
            r.req.path.id_or_action,
            r.req.path.param1,
            r.req.path.id_or_action2,
            r.req.path.param2,
        ) {
            (Some("_db"), Some("_all"), ..) => handlers::get_all_dbs(),
            (Some("_db"), Some(cf), Some("_since"), Some(id), Some("_topic"), topic) => {
                if id.ends_with('*') {
                    handlers::since_prefix_to_topic(SinceRequest {
                        query: r.req.query,
                        id: Some(id.trim_end_matches('*')),
                        cf,
                        topic,
                        ch: r.req.ch,
                    })
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into()))
                } else {
                    handlers::since_to_topic(SinceRequest {
                        query: r.req.query,
                        id: r.req.path.param1,
                        cf,
                        topic,
                        ch: r.req.ch,
                    })
                }
            }
            (Some("_db"), Some(cf), Some("_since"), Some(id), ..) => {
                if id.ends_with('*') {
                    handlers::since_prefix(SinceRequest {
                        query: r.req.query,
                        id: Some(id.trim_end_matches('*')),
                        cf,
                        topic: None,
                        ch: r.req.ch,
                    })
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into()))
                } else {
                    handlers::since(SinceRequest {
                        query: r.req.query,
                        id: r.req.path.param1,
                        cf,
                        topic: None,
                        ch: r.req.ch,
                    })
                }
            }
            (Some("_db"), Some(cf_name), Some(id), ..) => {
                match id {
                    "_all" => handlers::all(r.req.query, cf_name),
                    "_all_reverse" => handlers::all_reverse(r.req.query, cf_name),
                    id => {
                        handlers::get(r.db, cf_name, id, r.req.query)
                            .and_then(Ok)
                            .or_else(|err| Ok(err.into()))
                    }
                }
            }
            _ => Err(Error::WrongQuery),
        }
    }

    fn get_channel(&self, query: &Option<Query>) -> Result<Option<Channel>, Error> {
        let inner = self.db.clone();

        if let Some(channel_id) = query.as_ref().and_then(|q| q.channel.as_ref()) {
            let res = rocks::get_with_db(&inner.read().unwrap(), "_channel", &channel_id)?
                .next()
                .ok_or_else(|| Error::ChannelNotFound(channel_id.to_string()))?;
            let c = Channel::new_vec(res.value)?;
            return Ok(Some(c));
        }

        Ok(None)
    }
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

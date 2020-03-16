extern crate hyper;
extern crate tokio;

use std::env;
use std::sync::{Arc, RwLock};
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
            Method::GET => self.get_handlers(common),
            Method::PUT => self.put_handlers(common),
            Method::POST => self.post_handlers(common),
            _ => Err(Error::MethodNotFound),
        };

        match res {
            Ok(res) => future::ok(res),
            Err(err) => future::ok(err.into()),
        }
    }
}

impl Svc {
    fn put_handlers(&self, req: CommonRequest<'_>) -> Result<Response<Body>, Error> {
        match (req.path.route, req.path.cf, req.path.id_or_action) {
            // (Some("_db"), Some(cf), Some("_create_secondary_index"))=>Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
            (Some("_db"), Some(cf), Some("_create_db")) => handlers::create_db(self.db.clone(), cf),
            (Some("_db"), Some(cf), path_id) => {
                handlers::put(PutRequest {
                    cf,
                    query: req.query,
                    path_id,
                    req: req.body,
                    ch: req.ch,
                    db: self.db.clone(),
                })
                .and_then(Ok)
                .or_else(|err| Ok(err.into()))
            }
            _ => Err(Error::WrongQuery),
        }
    }

    fn post_handlers(&self, req: CommonRequest<'_>) -> Result<Response<Body>, Error> {
        match (req.path.route, req.path.cf, req.path.id_or_action) {
            (Some("_sql"), ..) => handlers::sql(self.db.clone(), req.query, req.body, req.ch),
            (Some("_db"), Some(cf_name), Some(id)) => {
                handlers::get(self.db.clone(), cf_name, id, req.query, req.ch)
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into()))
            }
            _ => Err(Error::WrongQuery),
        }
    }

    fn get_handlers(&self, r: CommonRequest<'_>) -> Result<Response<Body>, Error> {
        match (
            r.path.route,
            r.path.cf,
            r.path.id_or_action,
            r.path.param1,
            r.path.id_or_action2,
            r.path.param2,
        ) {
            (Some("_db"), Some("_all"), ..) => handlers::get_all_dbs(),
            (Some("_db"), Some(cf), Some("_since"), Some(id), Some("_topic"), topic) => {
                let is_prefix = id.ends_with('*');
                let since_request = SinceRequest {
                    query: r.query,
                    id: if is_prefix {
                        Some(id.trim_end_matches('*'))
                    } else {
                        r.path.param1
                    },
                    cf,
                    topic,
                    ch: r.ch,
                    db: self.db.clone(),
                };

                if is_prefix {
                    handlers::since_prefix(since_request)
                } else {
                    handlers::since(since_request)
                }
            }
            (Some("_db"), Some(cf_name), Some(id), ..) => {
                match id {
                    "_all" | "_all_reverse" => {
                        handlers::all(self.db.clone(), r.query, cf_name, r.ch)
                    }
                    id => {
                        handlers::get(self.db.clone(), cf_name, id, r.query, r.ch)
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
            let res = rocks::get(inner, "_channel", &channel_id, |x| vec![x])?
                .pop()
                .ok_or_else(|| Error::ChannelNotFound(channel_id.to_string()))?;

            let c = Channel::new_vec(
                res.value,
                query
                    .as_ref()
                    .and_then(|q| q.omit_errors)
                    .unwrap_or_default(),
            )?;
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

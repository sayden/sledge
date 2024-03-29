use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::service::Service;
use hyper::{Body, Request, Response};

use crate::channels::channel::Channel;
use crate::components::errors::Error;
use crate::components::rocks;
use crate::server::handlers;
use crate::server::handlers::{AppRequest, PutRequest, SPath, SinceRequest, SqlRequest};
use crate::server::query::Query;

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

        let ch = match self.fetch_channel(&query) {
            Ok(res) => res,
            Err(err) => return future::ok(err.into()),
        };

        let common = AppRequest {
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
    pub fn new(db: Arc<RwLock<rocksdb::DB>>) -> Self {
        Svc { db }
    }

    fn put_handlers(&self, req: AppRequest<'_>) -> Result<Response<Body>, Error> {
        match (req.path.route, req.path.cf, req.path.id_or_action) {
            // (Some("_db"), Some(cf), Some("_create_secondary_index"))=>Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
            (Some("_db"), Some(cf), Some("_create_db")) => handlers::create_db(self.db.clone(), cf),
            (Some("_db"), Some(cf), id) => {
                handlers::put(PutRequest::new(self.db.clone(), req, cf, id))
            }
            _ => Err(Error::WrongQuery),
        }
        .and_then(Ok)
        .or_else(|err| Ok(err.into()))
    }

    fn post_handlers(&self, r: AppRequest<'_>) -> Result<Response<Body>, Error> {
        match (r.path.route, r.path.cf, r.path.id_or_action) {
            (Some("_sql"), ..) => {
                handlers::sql(SqlRequest::new(self.db.clone(), r.query, r.body, r.ch))
            }
            (Some("_db"), Some(cf), Some(id)) => {
                handlers::get(self.db.clone(), cf, id, r.query, r.ch)
            }
            (Some("_test"), ..) => handlers::try_streaming(self.db.clone()),

            _ => Err(Error::WrongQuery),
        }
        .and_then(Ok)
        .or_else(|err| Ok(err.into()))
    }

    fn get_handlers(&self, r: AppRequest<'_>) -> Result<Response<Body>, Error> {
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
                let since_request = SinceRequest::new(self.db.clone(), r, id, cf, topic);
                handlers::since(since_request)
            }
            (Some("_db"), Some(cf_name), Some(id), ..) => match id {
                "_all" | "_all_reverse" => handlers::all(self.db.clone(), r.query, cf_name, r.ch),
                id => handlers::get(self.db.clone(), cf_name, id, r.query, r.ch),
            },
            _ => Err(Error::WrongQuery),
        }
        .and_then(Ok)
        .or_else(|err| Ok(err.into()))
    }

    fn fetch_channel(&self, query: &Option<Query>) -> Result<Option<Channel>, Error> {
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

fn get_query(uri: &Uri) -> Option<Query> {
    serde_urlencoded::from_str::<Query>(uri.query()?).ok()
}

fn get_path(p: &str) -> Vec<&str> {
    //TODO use some library for this
    p.split('/').filter(|x| x != &"").collect()
}

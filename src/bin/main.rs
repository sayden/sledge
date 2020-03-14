extern crate hyper;
extern crate tokio;

use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::service::Service;
use hyper::{Body, Request, Response, Server};

use sledge::components::errors::Error;
use sledge::server::handlers;
use sledge::server::query::Query;

fn get_query(uri: &Uri) -> Option<Query> {
    serde_urlencoded::from_str::<Query>(uri.query()?).ok()
}

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

struct ReadRequest<'a> {
    path: SPath<'a>,
    query: Option<Query>,
}

struct BodyRequest<'a> {
    path: SPath<'a>,
    query: Option<Query>,
    body: Body,
}

pub struct MakeSvc;

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future {
        future::ok(Svc {})
    }
}

#[derive(Debug)]
pub struct Svc {}

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

        let res: Result<Response<Body>, Error> = match parts.method {
            Method::GET => get_handlers(ReadRequest { path, query }),
            Method::PUT => put_handlers(BodyRequest { path, query, body }),
            Method::POST => post_handlers(BodyRequest { path, query, body }),
            _ => Err(Error::MethodNotFound),
        };

        match res {
            Ok(res) => future::ok(res),
            Err(err) => future::ok(err.into()),
        }
    }
}

fn post_handlers(req: BodyRequest<'_>) -> Result<Response<Body>, Error> {
    match (req.path.route, req.path.cf, req.path.id_or_action) {
        (Some("_sql"), _, _) => handlers::sql(req.query, req.body),
        (Some("_db"), Some(cf_name), Some(id)) => handlers::get(cf_name, id, req.query)
            .and_then(Ok)
            .or_else(|err| Ok(err.into())),
        _ => Err(Error::WrongQuery),
    }
}

fn put_handlers(req: BodyRequest<'_>) -> Result<Response<Body>, Error> {
    match req.path.route {
        Some("_db") => match req.path.cf {
            Some(cf_name) => match req.path.id_or_action {
                // Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
                id => handlers::put(cf_name, &req.query, id, req.body)
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into())),
            },
            _ => Err(Error::WrongQuery),
        },
        // (Some("db"), Some(cf_name), Some(id)) => match id {
        //     "_create_db" => handlers::create_db(cf_name),
        //     _ => Err(Error::WrongQuery)
        // },
        _ => Err(Error::WrongQuery),
    }
}

fn get_handlers(req: ReadRequest<'_>) -> Result<Response<Body>, Error> {
    match (
        req.path.route,
        req.path.cf,
        req.path.id_or_action,
        req.path.param1,
        req.path.id_or_action2,
        req.path.param2,
    ) {
        (Some("_db"), Some("_all"), ..) => handlers::get_all_dbs(),
        (
            Some("_db"),
            Some(cf_name),
            Some("_since"),
            Some(id),
            Some("_topic"),
            Some(topic_name),
        ) => {
            if id.ends_with('*') {
                handlers::since_prefix_to_topic(
                    req.query,
                    id.trim_end_matches('*'),
                    cf_name,
                    topic_name,
                )
                .and_then(Ok)
                .or_else(|err| Ok(err.into()))
            } else {
                handlers::since_to_topic(req.query, req.path.param1, cf_name, topic_name)
            }
        }
        (Some("_db"), Some(cf_name), Some("_since"), Some(id), ..) => {
            if id.ends_with('*') {
                handlers::since_prefix(req.query, id.trim_end_matches('*'), cf_name)
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into()))
            } else {
                handlers::since(req.query, req.path.param1, cf_name)
            }
        }
        (Some("_db"), Some(cf_name), Some(id), ..) => match id {
            "_all" => handlers::all(req.query, cf_name),
            "_all_reverse" => handlers::all_reverse(req.query, cf_name),
            id => handlers::get(cf_name, id, req.query)
                .and_then(Ok)
                .or_else(|err| Ok(err.into())),
        },
        _ => Err(Error::WrongQuery),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let addr = "127.0.0.1:1337".parse().unwrap();

    let server = Server::bind(&addr).serve(MakeSvc);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

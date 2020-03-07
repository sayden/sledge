#![feature(box_syntax)]

extern crate hyper;
extern crate tokio;

use std::convert::Infallible;
use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, Service, service_fn};

use sledge::channels::parser::Channel;
use sledge::components::errors::Error;
use sledge::components::rocks;
use sledge::server::handlers;
use sledge::server::query::Query;

fn get_query(uri: &Uri) -> Option<Query> {
    serde_urlencoded::from_str::<Query>(uri.query()?).ok()
}

fn get_path(p: &str) -> Vec<&str> {
    //TODO use some library for this
    p.split('/').filter(|x| x != &"").collect()
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

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let resp = http::Response::new(Body::from("ok"));
        future::ok(resp)
    }
}

struct SPath<'a> {
    route: Option<&'a str>,
    cf: Option<&'a str>,
    id: Option<&'a str>,
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

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (parts, body) = req.into_parts();
    let query = get_query(&parts.uri);
    let path = get_path(parts.uri.path());

    let path = SPath {
        route: path.get(0).cloned(),
        cf: path.get(1).cloned(),
        id: path.get(2).cloned(),
    };

    let res = match parts.method {
        Method::GET => {
            get_handlers(ReadRequest {
                path,
                query,
            })
                .await
        }
        Method::PUT => {
            put_handlers(BodyRequest {
                path,
                query,
                body,
            })
                .await
        }
        Method::POST => {
            post_handlers(BodyRequest {
                path,
                query,
                body,
            })
                .await
        }
        _ => Err(Error::MethodNotFound),
    };

    match res {
        Ok(res) => Ok(res),
        Err(err) => Ok(err.into()),
    }
}

async fn post_handlers(req: BodyRequest<'_>) -> Result<Response<Body>, Error> {
    match (req.path.route, req.path.cf, req.path.id) {
        (Some("_db"), Some(cf_name), Some(id)) => {
            match id {
                "_all" => handlers::range(req.query, None, cf_name).await,
                "_since" => handlers::range(req.query, None, cf_name).await,
                id => handlers::get(cf_name, id)
                    .and_then(Ok)
                    .or_else(|err| Ok(err.into())),
            }
        }
        _ => Err(Error::WrongQuery),
    }
}

async fn put_handlers(req: BodyRequest<'_>) -> Result<Response<Body>, Error> {
    match req.path.route {
        Some("_db") => match req.path.cf {
            Some(cf_name) => match req.path.id {
                // Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
                id => handlers::put(cf_name, &req.query, id, req.body)
                    .await
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

async fn get_handlers(req: ReadRequest<'_>) -> Result<Response<Body>, Error> {
    match (req.path.route, req.path.cf, req.path.id) {
        (Some("_db"), Some(cf_name), Some(id)) => match id {
            "_all" => handlers::range(req.query, None, cf_name).await,
            "_since" => handlers::range(req.query, None, cf_name).await,
            id => {
                if id.ends_with('*') {
                    handlers::range_prefix(req.query, id.trim_end_matches('*'), cf_name).await
                        .and_then(Ok)
                        .or_else(|err| Ok(err.into()))
                } else {
                    handlers::get(cf_name, id)
                        .and_then(Ok)
                        .or_else(|err| Ok(err.into()))
                }
            }
        },
        _ => Err(Error::WrongQuery),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(router)) });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

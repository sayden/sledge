#![feature(box_syntax)]

extern crate hyper;
extern crate tokio;

use std::convert::Infallible;
use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::service::{make_service_fn, service_fn, Service};
use hyper::{Body, Request, Response, Server};
use serde_urlencoded;

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
    p.split("/").filter(|x| x != &"").collect()
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

struct GetRequest<'a> {
    path: SPath<'a>,
    channel: Option<Channel>,
    query: Option<Query>,
}

struct PRequest<'a> {
    path: SPath<'a>,
    channel: Option<Channel>,
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

    let channel = match get_channel(&query) {
        Ok(res) => res,
        Err(err) => return Ok(err.into()),
    };

    let path = SPath {
        route: path.get(0).cloned(),
        cf: path.get(1).cloned(),
        id: path.get(2).cloned(),
    };

    let res = match parts.method {
        Method::GET => {
            method_get_handlers(GetRequest {
                path,
                channel,
                query,
            })
            .await
        }
        Method::PUT => {
            method_put_handlers(PRequest {
                path,
                query,
                channel,
                body,
            })
            .await
        }
        Method::POST => {
            method_post_handlers(PRequest {
                path,
                query,
                channel,
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

async fn method_post_handlers(req: PRequest<'_>) -> Result<Response<Body>, Error> {
    return match (req.path.route, req.path.cf, req.path.id) {
        (Some("_db"), Some(cf_name), Some(id)) => {
            let maybe_post_channel = match get_channel_or_err(req.body).await {
                Ok(ch) => Some(ch),
                Err(e) => return Ok(From::from(e)),
            }
            .or(req.channel);

            match id {
                "_all" => handlers::range(req.query, None, cf_name, maybe_post_channel).await,
                "_since" => handlers::range(req.query, None, cf_name, maybe_post_channel).await,
                id => handlers::get(cf_name, id, &maybe_post_channel)
                    .and_then(|res| Ok(res))
                    .or_else(|err| Ok(err.into())),
            }
        }
        _ => Err(Error::WrongQuery),
    };
}

async fn method_put_handlers(req: PRequest<'_>) -> Result<Response<Body>, Error> {
    return match req.path.route {
        Some("_db") => match req.path.cf {
            Some(cf_name) => match req.path.id {
                Some("_create_secondary_index") => handlers::create(req.query, cf_name).await,
                id => handlers::put(cf_name, &req.query, id, req.body, &req.channel)
                    .await
                    .and_then(|res| Ok(res))
                    .or_else(|err| Ok(err.into())),
            },
            _ => Err(Error::WrongQuery),
        },
        // (Some("db"), Some(cf_name), Some(id)) => match id {
        //     "_create_db" => handlers::create_db(cf_name),
        //     _ => Err(Error::WrongQuery)
        // },
        _ => Err(Error::WrongQuery),
    };
}

async fn method_get_handlers(req: GetRequest<'_>) -> Result<Response<Body>, Error> {
    return match (req.path.route, req.path.cf, req.path.id) {
        (Some("_db"), Some(cf_name), Some(id)) => match id {
            "_all" => return handlers::range(req.query, None, cf_name, req.channel).await,
            "_since" => return handlers::range(req.query, None, cf_name, req.channel).await,
            id => handlers::get(cf_name, id, &req.channel)
                .and_then(|res| Ok(res))
                .or_else(|err| Ok(err.into())),
        },
        (Some("_db"), Some("_all"), _) => handlers::get_all_dbs(),
        _ => Err(Error::WrongQuery),
    };
}

async fn get_channel_or_err(body: Body) -> Result<Channel, Error> {
    let whole_body = match hyper::body::to_bytes(body).await {
        Err(err) => return Err(Error::BodyParsingError(err)),
        Ok(body) => body,
    };

    Channel::new_u8(whole_body.as_ref())
}

fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error> {
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            Some(channel_id) => {
                let res = rocks::get("_channel", &channel_id)?;
                let c = Channel::new_vec(res)?;
                return Ok(Some(c));
            }
            None => Ok(None),
        },
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

#![feature(box_syntax)]

extern crate hyper;
extern crate tokio;

use std::convert::Infallible;
use std::task::{Context, Poll};

use futures_util::future;
use http::{Method, Uri};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, Service, service_fn};
use serde_urlencoded;

use sledge::channels::parser::Channel;
use sledge::components::rocks;
use sledge::components::errors::Error;
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
    maybe_route: Option<&'a str>,
    maybe_cf: Option<&'a str>,
    maybe_id: Option<&'a str>,
}

struct GetRequest<'a> {
    path: SPath<'a>,
    maybe_channel: Option<Channel>,
    maybe_query: Option<Query>,
}

struct PRequest<'a> {
    path: SPath<'a>,
    maybe_channel: Option<Channel>,
    maybe_query: Option<Query>,
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
    let maybe_query = get_query(&parts.uri);
    let path = get_path(parts.uri.path());

    let maybe_channel = match get_channel(&maybe_query) {
        Ok(res) => res,
        Err(err) => return Ok(err.into()),
    };

    let spath = SPath {
        maybe_route: path.get(0).cloned(),
        maybe_cf: path.get(1).cloned(),
        maybe_id: path.get(2).cloned(),
    };

    let res = match parts.method {
        Method::GET => method_get_handlers(GetRequest { path: spath, maybe_channel, maybe_query }).await,
        Method::PUT => method_put_handlers(PRequest { path: spath, maybe_query, maybe_channel, body }).await,
        Method::POST => method_post_handlers(PRequest { path: spath, maybe_query, maybe_channel, body }).await,
        _ => Err(Error::MethodNotFound),
    };

    match res {
        Ok(res) => Ok(res),
        Err(err) => Ok(err.into())
    }
}

async fn method_post_handlers(req: PRequest<'_>) -> Result<Response<Body>, Error> {
    return match (req.path.maybe_route, req.path.maybe_cf, req.path.maybe_id) {
        (Some("db"), Some(cf_name), Some(id)) => {
            let maybe_post_channel = match get_channel_or_err(req.body).await {
                Ok(ch) => Some(ch),
                Err(e) => return Ok(From::from(e)),
            }.or(req.maybe_channel);

            match id {
                "_all" => handlers::range(req.maybe_query, None, cf_name, maybe_post_channel).await,
                "_since" => handlers::range(req.maybe_query, None, cf_name, maybe_post_channel).await,
                "_create" => handlers::range(req.maybe_query, None, cf_name, maybe_post_channel).await,
                id => handlers::get(cf_name, id, &maybe_post_channel)
                    .and_then(|res| Ok(res)).or_else(|err| Ok(err.into())),
            }
        }
        _ => Err(Error::WrongQuery),
    };
}

async fn method_put_handlers(req: PRequest<'_>) -> Result<Response<Body>, Error> {
    return match (req.path.maybe_route, req.path.maybe_cf) {
        (Some("db"), Some(cf_name)) =>
            handlers::put(cf_name, &req.maybe_query, req.path.maybe_id, req.body, &req.maybe_channel).await
                .and_then(|res| Ok(res)).or_else(|err| Ok(err.into())),
        _ => Err(Error::WrongQuery),
    };
}

async fn method_get_handlers(req: GetRequest<'_>) -> Result<Response<Body>, Error> {
    return match (req.path.maybe_route, req.path.maybe_cf, req.path.maybe_id) {
        (Some("db"), Some(cf_name), Some(id)) => {
            match id {
                "_all" => return handlers::range(req.maybe_query, None, cf_name, req.maybe_channel).await,
                "_since" => return handlers::range(req.maybe_query, None, cf_name, req.maybe_channel).await,
                id => handlers::get(cf_name, id, &req.maybe_channel)
                    .and_then(|res| Ok(res)).or_else(|err| Ok(err.into())),
            }
        }
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

fn get_channel(maybe_query: &Option<Query>) -> Result<Option<Channel>, Error>
{
    match maybe_query {
        None => Ok(None),
        Some(query) => match &query.channel {
            Some(channel_id) => {
                let res = rocks::get("_channel", &channel_id)?;
                let c = Channel::new_vec(res)?;
                return Ok(Some(c));
            }
            None => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(router)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

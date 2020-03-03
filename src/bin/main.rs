#![feature(box_syntax)]

extern crate hyper;
extern crate tokio;

use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::net::TcpListener as StdTcpListener;
use std::pin::Pin;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::{Future, Stream};
use futures_util::future;
use http::{Method, Uri};
use hyper::{Body, Request, Response, Server, StatusCode, Version};
use hyper::server::conn::Http;
use hyper::service::{make_service_fn, Service, service_fn};
use rocksdb::{IteratorMode, Options};
use serde_json::Value;
use serde_urlencoded;
use tokio::net::{TcpListener, TcpStream as TkTcpStream};
use uuid::Uuid;

use sledge::components::{rocks, handlers};
use sledge::components::storage::{Error, IterMod};
use sledge::server::query::Query;

fn get_query(uri: &Uri) -> Option<Query> {
    serde_urlencoded::from_str::<Query>(uri.query()?).ok()
}

fn get_path(p: &str) -> Vec<&str> {
    //TODO use some library for this
    p.split("/").filter(|x| x != &"").collect()
}

// #[derive(Debug)]
// pub struct Svc {}
//
// impl Service<Request<Body>> for Svc {
//     type Response = Response<Body>;
//     type Error = hyper::Error;
//     type Future = future::Ready<Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Ok(()).into()
//     }
//
//     fn call(&mut self, req: Request<Body>) -> Self::Future {
//         let query = get_query(req.uri());
//         let path = get_path(req.uri().path());
//
//         match req.method() {
//             (&Method::GET) => {
//                 match path.first() {
//                     Some(&"db") => match path.get(1) {
//                         Some(&cf_name) => match path.get(2) {
//                             Some(&id) => match id {
//                                 "_all" => return rocks::range(query, None, cf_name),
//                                 id => return match query {
//                                     Some(q) => {
//                                         if q.limit.is_some() || q.skip.is_some() || q.direction_reverse.is_some() || q.until_key.is_some() {
//                                             rocks::range(Some(q), Some(id), cf_name)
//                                         } else {
//                                             handlers::get(cf_name.to_string(), id.to_string())
//                                         }
//                                     }
//                                     None => handlers::get(cf_name.to_string(), id.to_string())
//                                 }
//                             },
//                             None => println!("statistics about the cf"),
//                         },
//                         None => println!("statistics about the db"),
//                     },
//                     _ => println!("welcome page"),
//                 }
//             }
//             (&Method::PUT) => {
//                 match path.first() {
//                     Some(&"db") => match path.get(1) {
//                         Some(&cf_name) => match path.get(2) {
//                             Some(&id) => match id {
//                                 "_auto" => return handlers::put(cf_name.to_string(), query, None, req.into_body()),
//                                 id => return handlers::put(cf_name.to_string(), query, Some(id), req.into_body()),
//                             }
//                             None => println!("no id found"),
//                         }
//                         _ => println!("you must specify a cf"),
//                     }
//                     _ => println!("root path not recognized")
//                 }
//             }
//             _ => println!("method not recognized"),
//         }
//
//         let resp = http::Response::new(Body::from("ok"));
//         future::ok(resp)
//     }
// }
//
// pub struct MakeSvc;
//
// impl<T> Service<T> for MakeSvc {
//     type Response = Svc;
//     type Error = std::io::Error;
//     type Future = future::Ready<Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Ok(()).into()
//     }
//
//     fn call(&mut self, _: T) -> Self::Future {
//         future::ok(Svc {})
//     }
// }

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (parts, body) = req.into_parts();
    let query = get_query(&parts.uri);
    let path = get_path(parts.uri.path());

    match parts.method {
        Method::GET => {
            match path.first() {
                Some(&"db") => match path.get(1) {
                    Some(&cf_name) => match path.get(2) {
                        Some(&id) => match id {
                            "_all" => return rocks::range(query, None, cf_name),
                            id => return match query {
                                Some(q) => {
                                    if q.limit.is_some() || q.skip.is_some() || q.direction_reverse.is_some() || q.until_key.is_some() {
                                        rocks::range(Some(q), Some(id), cf_name)
                                    } else {
                                        handlers::get(cf_name.to_string(), id.to_string())
                                    }
                                }
                                None => handlers::get(cf_name.to_string(), id.to_string())
                            }
                        },
                        None => println!("statistics about the cf"),
                    },
                    None => println!("statistics about the db"),
                },
                _ => println!("welcome page"),
            }
        }
        Method::PUT => {
            match path.first() {
                Some(&"db") => match path.get(1) {
                    Some(&cf_name) => match path.get(2) {
                        Some(&id) => match id {
                            "_auto" => return handlers::put(cf_name.to_string(), query, None, body).await,
                            id => return handlers::put(cf_name.to_string(), query, Some(id), body).await,
                        }
                        None => {
                            return handlers::put(cf_name.to_string(), query, None, body).await
                        },
                    }
                    _ => println!("you must specify a cf"),
                }
                _ => println!("root path not recognized")
            }
        }
        _ => println!("method not recognized"),
    }

    let resp = http::Response::new(Body::from("ok"));
    // future::ok(resp)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(router)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

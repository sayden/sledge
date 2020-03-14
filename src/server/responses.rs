use bytes::Bytes;
use hyper::Body;
use hyper::Response;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::components::errors::Error;
use crate::components::iterator::SledgeIterator;
use crate::components::simple_pair::{simple_pair_to_json, KvUTF8};
use crate::server::handlers::{BytesResultIterator, BytesResultStream};
use crate::server::query::Query;
use crate::server::reply::Reply;
use futures::executor::block_on;

pub fn new_read_ok(res: &[u8]) -> Result<Response<Body>, Error> {
    let data: Box<Value> = box serde_json::from_slice(res).map_err(Error::SerdeError)?;
    let reply = Reply::ok(Some(data));
    Ok(reply.into())
}

pub fn new_read_ok_iter(iter: SledgeIterator) -> Result<Response<Body>, Error> {
    let data = box serde_json::to_value(
        iter.flat_map(|x| simple_pair_to_json(x, true))
            .collect::<Vec<Value>>(),
    )
    .map_err(Error::SerdeError)?;

    let reply = Reply::ok(Some(data));

    Ok(reply.into())
}

pub fn get_iterating_response(
    iter: SledgeIterator,
    query: Option<Query>,
) -> Result<Response<Body>, Error> {
    let include_id = query.and_then(|q| q.include_ids).unwrap_or_else(|| false);

    let thread_iter: Box<BytesResultIterator> = box iter
        .flat_map(move |x| simple_pair_to_json(x, include_id))
        .flat_map(|spj| {
            serde_json::to_string(&spj)
                .map_err(|err| {
                    log::warn!(
                        "error trying to get json from simpleJSON: {}",
                        err.to_string()
                    )
                })
                .ok()
        })
        .map(|s| format!("{}\n", s))
        .map(|x| Ok(Bytes::from(x)));

    let stream: BytesResultStream = box futures::stream::iter(thread_iter);

    http::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(stream))
        .map_err(Error::GeneratingResponse)
}

#[derive(Serialize, Deserialize)]
struct TotalRecords {
    total_records: i32,
}

pub fn get_iterating_response_with_topic(
    iter: SledgeIterator,
    topic_name: &str,
) -> Result<Response<Body>, Error> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "4af3c87b16f6:9092")
        .set("message.timeout.ms", "5000")
        .create()
        .map_err(Error::KafkaError)?;

    let thread_iter = iter.filter_map(From::from).map(|v: KvUTF8| {
        producer.send(FutureRecord::to(topic_name).payload(&v.value).key(&v.id), 0)
    });

    let mut reply = Reply::empty();
    let mut records = TotalRecords { total_records: 0 };

    for delivery_result in thread_iter {
        records.total_records += 1;

        let result = block_on(delivery_result);
        match result {
            Ok(r) => {
                if let Err(err) = r {
                    reply.error = true;
                    reply.cause = reply
                        .cause
                        .map(|s| format!("{}: {:?}. {}", err.0.to_string(), err.1, s));
                }
            }
            Err(_) => {
                reply.error = true;
                reply.cause = reply.cause.map(|s| format!("Cancelled delivery. {}", s));
            }
        }
    }

    reply.data = Some(box serde_json::to_value(records).unwrap_or_default());

    Ok(reply.into())
}

pub fn unknown_error(err: String) -> Response<Body> {
    http::Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(format!(
            r#"{{"result":{{"error":"true", "cause":"{}"}}}}"#,
            err
        )))
        .unwrap()
}

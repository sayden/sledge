use std::str::from_utf8;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use chrono::Utc;
use futures::executor::block_on;
use futures::Stream;
use http::Response;
use hyper::Body;
use rocksdb::DBIterator;
use serde_json::Value;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use uuid::Uuid;

use crate::channels::channel::Channel;
use crate::components::errors::Error;
use crate::components::raw_iterator::RawIteratorWrapper;
use crate::components::rocks;
use crate::components::rocks::SledgeIterator;
use crate::components::simple_pair::{simple_pair_to_json, SimplePair};
use crate::components::sql;
use crate::server::filters::Filters;
use crate::server::query::Query;
use crate::server::reply::Reply;
use crate::server::responses::get_iterating_response_with_topic;

pub type BytesResult = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;
pub type BytesResultStream = Box<dyn Stream<Item = BytesResult> + Send + Sync>;
pub type BytesResultIterator = dyn Iterator<Item = BytesResult> + Send + Sync;

// struct IndexedValue {
//     key: String,
//     value: String,
// }

// pub async fn create(query: Option<Query>, cf_name: &str) -> Result<Response<Body>, Error> {
//     let id = query
//         .clone()
//         .map(|q| q.field_path)
//         .flatten()
//         .ok_or(Error::MissingID)?;
//
//     let iter: SledgeIterator = rocks::range(query, None, cf_name, None)?;
//
//     let new_cf_name = format!("{}_by_{}", cf_name, id);
//
//     let final_iter = iter
//         .flat_map(|x| (x.k, serde_json::to_value(x.v)
//             .map_err(|err| log::warn!("error trying to get json from value: {}", err.to_string()))
//             .ok()?))
//         .flat_map(|x| {
//             json_nested_value(&id, x.1)
//                 .as_str()
//                 .map(|k| IndexedValue {
//                     key: format!("{}{}", k, x.0),
//                     value: format!("\"{}\"", x.1),
//                 })
//         })
//         .flat_map(
//             |x| match rocks::put(&new_cf_name, &x.key, Bytes::from(x.value)) {
//                 Ok(()) => None,
//                 Err(err) => Some(err.to_string()),
//             },
//         );
//
//     let res = final_iter.fold((0, "".to_string()), |acc, err| {
//         (acc.0 + 1, format!("{}, {}", acc.1, err))
//     });
//
//     Ok(if res.0 > 0 {
//         WriteReply::<&str> {
//             result: ResultEmbeddedReply::error(res.1.as_str()),
//             id: None,
//         }
//     } else {
//         WriteReply::<&str> {
//             result: ResultEmbeddedReply::ok(),
//             id: None,
//         }
//     }
//         .into())
// }

pub struct SinceRequest<'a> {
    pub query: Option<Query>,
    pub id: Option<&'a str>,
    pub cf: &'a str,
    pub topic: Option<&'a str>,
    pub ch: Option<Channel>,
    pub db: Arc<RwLock<rocksdb::DB>>,
}

pub struct PutRequest<'a> {
    pub cf: &'a str,
    pub query: Option<Query>,
    pub path_id: Option<&'a str>,
    pub req: Body,
    pub ch: Option<Channel>,
    pub db: Arc<RwLock<rocksdb::DB>>,
}

pub fn since_prefix(r: SinceRequest) -> Result<Response<Body>, Error> {
    let data = rocks::range_prefix(r.db, r.id.unwrap(), r.cf, filters_raw(r.query, r.ch))?;

    get_iterating_response_with_topic(data, r.topic)
}

pub fn since(r: SinceRequest) -> Result<Response<Body>, Error> {
    let id = get_id(&r.query, r.id, None)?;

    let data = rocks::range(
        r.db,
        is_reverse(&r.query),
        Some(id.as_ref()),
        r.cf,
        filters(r.query, r.ch),
    )?;

    get_iterating_response_with_topic(data, r.topic)
}

pub fn all(
    db: Arc<RwLock<rocksdb::DB>>,
    query: Option<Query>,
    cf: &str,
    ch: Option<Channel>,
) -> Result<Response<Body>, Error> {
    let id = get_id(&query, None, None)?;
    let result = rocks::range(
        db,
        is_reverse(&query),
        Some(id.as_str()),
        cf,
        filters(query, ch),
    )?;
    new_read_ok_iter_with_db(result)
}

pub fn put(r: PutRequest) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(r.req)).map_err(Error::BodyParsingError)?;
    let id = get_id(&r.query, r.path_id, Some(&value))?;

    let cf = r.cf;
    let mut filters = Filters::new(r.query, r.ch, None);
    let sp = SimplePair {
        id: Vec::from(id),
        value: value.to_vec(),
    };
    let iter = filters.apply(vec![sp].into_iter());
    let db = r.db;

    let errors = iter
        .filter_map(|v| {
            let res = rocks::put(db.clone(), cf, v.id, v.value);
            if let Err(e) = res {
                Some(e.to_string())
            } else {
                None
            }
        })
        .fold(None, |acc, s| {
            match acc {
                Some(e) => Some(format!("{}, {}", e, s)),
                None => Some(s),
            }
        });

    match errors {
        Some(errs) => Err(Error::Multi(errs)),
        None => Ok(Reply::ok(None).into()),
    }
}

pub fn sql(
    db: Arc<RwLock<rocksdb::DB>>,
    query: Option<Query>,
    req: Body,
    ch: Option<Channel>,
) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(req)).map_err(Error::BodyParsingError)?;
    let sql = from_utf8(value.as_ref()).map_err(|err| Error::Utf8Error(err.to_string()))?;

    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string()).map_err(Error::SqlError)?;

    let from = sql::utils::get_from(&ast).ok_or_else(|| Error::CFNotFound("".to_string()))?;

    let result = rocks::range(
        db,
        is_reverse(&query),
        None,
        from.as_str(),
        filters(query, ch),
    )?;
    new_read_ok_iter_with_db(result)
}

pub fn get(
    db: Arc<RwLock<rocksdb::DB>>,
    cf: &str,
    id: &str,
    query: Option<Query>,
    ch: Option<Channel>,
) -> Result<Response<Body>, Error> {
    let result = rocks::get(db, &cf, &id, filters_single(query, ch))?;
    new_read_ok_iter_with_db(result)
}

pub fn get_all_dbs() -> Result<Response<Body>, Error> {
    let res = rocks::get_all_dbs()?;

    let v = serde_json::to_string(&res).map_err(Error::SerdeError)?;

    let data: Box<Value> = box serde_json::from_slice(v.as_bytes()).map_err(Error::SerdeError)?;
    let reply = Reply::ok(Some(data));
    Ok(reply.into())
}

pub fn create_db(db: Arc<RwLock<rocksdb::DB>>, cf: &str) -> Result<Response<Body>, Error> {
    if let Err(err) = rocks::create_cf(db, cf) {
        return Ok(err.into());
    }

    Ok(Reply::ok(None).into())
}

fn is_reverse(q: &Option<Query>) -> bool {
    q.as_ref()
        .and_then(|q| q.direction_reverse)
        .unwrap_or_default()
}

pub fn new_read_ok_iter_with_db(v: Vec<SimplePair>) -> Result<Response<Body>, Error> {
    let data = box serde_json::to_value(
        v.into_iter()
            .flat_map(|x| simple_pair_to_json(x, true))
            .collect::<Vec<Value>>(),
    )
    .map_err(Error::SerdeError)?;

    let reply = Reply::ok(Some(data));

    Ok(reply.into())
}

fn filters(
    query: Option<Query>,
    ch: Option<Channel>,
) -> Box<dyn FnOnce(DBIterator) -> Vec<SimplePair>> {
    box move |iter| -> Vec<SimplePair> {
        let sledge_iter = iter.map(SimplePair::new_boxed);
        let mut mods = Filters::new(query, ch, None);
        let iter2 = mods.apply(sledge_iter);

        iter2.collect::<Vec<_>>()
    }
}

fn filters_raw(
    query: Option<Query>,
    ch: Option<Channel>,
) -> Box<dyn FnOnce(RawIteratorWrapper) -> Vec<SimplePair>> {
    box move |iter| -> Vec<SimplePair> {
        let mut mods = Filters::new(query, ch, None);
        let iter2 = mods.apply(iter);

        iter2.collect::<Vec<_>>()
    }
}

fn filters_single(
    query: Option<Query>,
    ch: Option<Channel>,
) -> Box<dyn FnOnce(SimplePair) -> Vec<SimplePair>> {
    box move |iter| -> Vec<SimplePair> {
        let mut mods = Filters::new(query, ch, None);
        let iter2 = mods.apply(vec![iter].into_iter());

        iter2.collect::<Vec<_>>()
    }
}

fn get_id(
    query: &Option<Query>,
    path_id: Option<&str>,
    req: Option<&Bytes>,
) -> Result<String, Error> {
    if let Some(q) = query {
        if let (Some(id), Some(req)) = (q.field_path.as_ref(), req) {
            let j: Value = serde_json::from_slice(req.as_ref()).map_err(Error::SerdeError)?;
            let val: &Value = json_nested_value(id, &j);
            return Ok(val
                .as_str()
                .ok_or_else(|| Error::IdNotFoundInJSON(id.clone()))?
                .to_string());
        }
    }

    let id = path_id.ok_or(Error::NoIdFoundOnRequest)?;
    match id {
        "_auto" => Ok(Uuid::new_v4().to_string()),
        "_auto_time" => Ok(Utc::now().to_rfc3339()),
        _ => Ok(id.to_string()),
    }
}

pub fn json_nested_value<'a>(k: &str, v: &'a Value) -> &'a Value {
    k.split('.').fold(v, move |acc, x| &acc[x])
}

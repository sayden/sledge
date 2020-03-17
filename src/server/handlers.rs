use std::sync::{Arc, RwLock};

use bytes::Bytes;
use chrono::Utc;
use futures::executor::block_on;
use http::Response;
use hyper::Body;
use rocksdb::DBIterator;
use serde_json::Value;
use sqlparser::{dialect::GenericDialect, parser::Parser};
use uuid::Uuid;

use crate::{
    channels::channel::Channel,
    components::{
        errors::Error,
        rocks,
        simple_pair::{simple_pair_to_json, SimplePair},
        sql,
    },
    server::{filters::Filters, query::Query, reply::Reply, responses::get_iterating_response_with_topic},
};

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
//             .map_err(|err| log::warn!("error trying to get json from value: {}",
// err.to_string()))             .ok()?))
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

pub struct AppRequest<'a> {
    pub ch:    Option<Channel>,
    pub path:  SPath<'a>,
    pub query: Option<Query>,
    pub body:  Body,
}

pub struct SPath<'a> {
    pub route: Option<&'a str>,
    pub cf:    Option<&'a str>,

    pub id_or_action: Option<&'a str>,
    pub param1:       Option<&'a str>,

    pub id_or_action2: Option<&'a str>,
    pub param2:        Option<&'a str>,
}

pub struct SinceRequest<'a> {
    pub query: Option<Query>,
    pub id:    Option<&'a str>,
    pub cf:    &'a str,
    pub topic: Option<&'a str>,
    pub ch:    Option<Channel>,
    pub db:    Arc<RwLock<rocksdb::DB>>,
    is_prefix: bool,
}

impl SinceRequest<'a> {
    pub fn new(
        db: Arc<RwLock<rocksdb::DB>>, req: AppRequest<'a>, id: &'a str, cf: &'a str, topic: Option<&'a str>,
    ) -> Self {
        let is_prefix = id.ends_with('*');
        let id = if is_prefix { Some(id.trim_end_matches('*')) } else { req.path.param1 };

        SinceRequest { query: req.query, id, cf, topic, ch: req.ch, db, is_prefix }
    }
}

pub struct SqlRequest {
    db:    Arc<RwLock<rocksdb::DB>>,
    query: Option<Query>,
    req:   Body,
    ch:    Option<Channel>,
}

impl SqlRequest {
    pub fn new(db: Arc<RwLock<rocksdb::DB>>, query: Option<Query>, req: Body, ch: Option<Channel>) -> Self {
        SqlRequest { db, query, req, ch }
    }
}

pub struct PutRequest<'a> {
    pub cf:      &'a str,
    pub query:   Option<Query>,
    pub path_id: Option<&'a str>,
    pub req:     Body,
    pub ch:      Option<Channel>,
    pub db:      Arc<RwLock<rocksdb::DB>>,
}

impl PutRequest<'a> {
    pub fn new(db: Arc<RwLock<rocksdb::DB>>, req: AppRequest, cf: &'a str, path_id: Option<&'a str>) -> Self {
        PutRequest { cf, query: req.query, path_id, req: req.body, ch: req.ch, db }
    }
}

pub fn since(r: SinceRequest) -> Result<Response<Body>, Error> {
    let id = get_id(&r.query, r.id, None)?;

    if r.is_prefix {
        let topic = r.topic;
        let data = rocks::range_prefix(r.db.clone(), id, r.cf, dbiterator_filters(r.query, r.ch))?;
        get_iterating_response_with_topic(data, topic)
    } else {
        let data = rocks::range(r.db, is_reverse(&r.query), id, r.cf, dbiterator_filters(r.query, r.ch))?;

        get_iterating_response_with_topic(data, r.topic)
    }
}

pub fn all(
    db: Arc<RwLock<rocksdb::DB>>, query: Option<Query>, cf: &str, ch: Option<Channel>,
) -> Result<Response<Body>, Error> {
    since(SinceRequest { query, id: None, cf, topic: None, ch, db, is_prefix: false })
}

pub fn sql(r: SqlRequest) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(r.req)).map_err(Error::BodyParsingError)?;
    let sql = std::str::from_utf8(value.as_ref()).map_err(|err| Error::Utf8Error(err.to_string()))?;

    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string()).map_err(Error::SqlError)?;

    let from = sql::utils::get_from(&ast).ok_or_else(|| Error::CFNotFound("".to_string()))?;

    since(SinceRequest {
        query:     r.query,
        id:        None,
        cf:        from.as_str(),
        topic:     None,
        ch:        r.ch,
        db:        r.db,
        is_prefix: false,
    })
}

pub fn try_streaming(db: Arc<RwLock<rocksdb::DB>>) -> Result<Response<Body>, Error> {
    let res = rocks::try_streaming(db, dbiterator_filters(None, None))?;
    new_read_ok_iter_with_db(res)
}

pub fn put(r: PutRequest) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(r.req)).map_err(Error::BodyParsingError)?;
    let id = get_id(&r.query, r.path_id, Some(&value))?;

    let cf = r.cf;
    let mut filters = Filters::new(r.query, r.ch, None);
    let sp = SimplePair { id: Vec::from(id), value: value.to_vec() };

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

pub fn get(
    db: Arc<RwLock<rocksdb::DB>>, cf: &str, id: &str, query: Option<Query>, ch: Option<Channel>,
) -> Result<Response<Body>, Error> {
    let result = rocks::get(db, &cf, &id, |i| {
        let iter = vec![i].into_iter();
        apply_filters(query, ch, iter)
    })?;

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
        return Ok(err.into())
    }

    Ok(Reply::ok(None).into())
}

fn is_reverse(q: &Option<Query>) -> bool { q.as_ref().and_then(|q| q.direction_reverse).unwrap_or_default() }

pub fn new_read_ok_iter_with_db(v: Vec<SimplePair>) -> Result<Response<Body>, Error> {
    let data =
        box serde_json::to_value(v.into_iter().flat_map(|x| simple_pair_to_json(x, true)).collect::<Vec<Value>>())
            .map_err(Error::SerdeError)?;

    let reply = Reply::ok(Some(data));

    Ok(reply.into())
}

fn dbiterator_filters(query: Option<Query>, ch: Option<Channel>) -> Box<dyn FnOnce(DBIterator) -> Vec<SimplePair>> {
    box move |iter| -> Vec<SimplePair> {
        let sledge_iter = iter.map(SimplePair::new_boxed);
        apply_filters(query, ch, sledge_iter)
    }
}

fn apply_filters(
    query: Option<Query>, ch: Option<Channel>, iter: impl Iterator<Item = SimplePair> + Send + Sync,
) -> Vec<SimplePair> {
    let mut mods = Filters::new(query, ch, None);
    let iter2 = mods.apply(iter);
    iter2.collect()
}

fn get_id(query: &Option<Query>, path_id: Option<&str>, req: Option<&Bytes>) -> Result<String, Error> {
    if let Some(q) = query {
        if let (Some(id), Some(req)) = (q.field_path.as_ref(), req) {
            let j: Value = serde_json::from_slice(req.as_ref()).map_err(Error::SerdeError)?;
            let val: &Value = json_nested_value(id, &j);
            return Ok(val.as_str().ok_or_else(|| Error::IdNotFoundInJSON(id.clone()))?.to_string())
        }
    }

    let id = path_id.ok_or(Error::NoIdFoundOnRequest)?;
    match id {
        "_auto" => Ok(Uuid::new_v4().to_string()),
        "_auto_time" => Ok(Utc::now().to_rfc3339()),
        _ => Ok(id.to_string()),
    }
}

pub fn json_nested_value<'a>(k: &str, v: &'a Value) -> &'a Value { k.split('.').fold(v, move |acc, x| &acc[x]) }

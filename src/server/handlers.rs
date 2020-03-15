use std::str::from_utf8;

use bytes::Bytes;
use chrono::Utc;
use futures::executor::block_on;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use uuid::Uuid;

use crate::channels::channel::Channel;
use crate::components::errors::Error;
use crate::components::iterator::{with_channel, with_channel_for_single_value, BoxedSledgeIter};
use crate::components::rocks;
use crate::components::rocks::SledgeIterator;
use crate::components::sql;
use crate::server::filters::Filters;
use crate::server::query::Query;
use crate::server::reply::Reply;
use crate::server::responses::get_iterating_response_with_topic;
use crate::server::responses::{get_iterating_response, new_read_ok, new_read_ok_iter};

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
}

pub fn since_prefix_to_topic(r: SinceRequest) -> Result<Response<Body>, Error> {
    let iter = rocks::range_prefix(r.id.unwrap(), r.cf)?;
    get_iterating_response_with_topic(after_read_actions(box iter, &r.query)?, r.topic)
}

pub fn since_prefix(r: SinceRequest) -> Result<Response<Body>, Error> {
    let iter = rocks::range_prefix(r.id.unwrap(), r.cf)?;
    get_iterating_response(after_read_actions(box iter, &r.query)?, r.query)
}

pub fn since(r: SinceRequest) -> Result<Response<Body>, Error> {
    let id = get_id(&r.query, r.id, None)?;
    let iter = rocks::range(&r.query, id.as_ref(), r.cf)?;
    get_iterating_response(after_read_actions(box iter, &r.query)?, r.query)
}

pub fn since_to_topic(r: SinceRequest) -> Result<Response<Body>, Error> {
    let id = get_id(&r.query, r.id, None)?;
    let iter = rocks::range(&r.query, id.as_ref(), r.cf)?;
    get_iterating_response_with_topic(after_read_actions(box iter, &r.query)?, r.topic)
}

pub fn all(query: Option<Query>, cf_name: &str) -> Result<Response<Body>, Error> {
    let iter = rocks::range_all(&query, None, cf_name)?;
    get_iterating_response(after_read_actions(box iter, &query)?, query)
}

pub fn all_reverse(query: Option<Query>, cf_name: &str) -> Result<Response<Body>, Error> {
    let iter = rocks::range_all_reverse(cf_name)?;
    get_iterating_response(after_read_actions(box iter, &query)?, query)
}

pub struct PutRequest<'a> {
    pub cf: &'a str,
    pub query: &'a Option<Query>,
    pub path_id: Option<&'a str>,
    pub req: Body,
}

pub fn put(r: PutRequest) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(r.req)).map_err(Error::BodyParsingError)?;
    let id = get_id(&r.query, r.path_id, Some(&value))?;

    let ch = get_channel(&r.query)?;
    let value = with_channel_for_single_value(value, ch, r.query);
    rocks::put(r.cf, &id, value)?;

    let res = Reply::ok(None);
    Ok(res.into())
}

pub fn sql(query: Option<Query>, req: Body) -> Result<Response<Body>, Error> {
    let value = block_on(hyper::body::to_bytes(req)).map_err(Error::BodyParsingError)?;
    let sql = from_utf8(value.as_ref()).map_err(|err| Error::Utf8Error(err.to_string()))?;

    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string()).map_err(Error::SqlError)?;

    let from = sql::utils::get_from(&ast).ok_or_else(|| Error::CFNotFound("".to_string()))?;

    let iter = rocks::range_all(&None, None, from.as_str())?;

    get_iterating_response(after_read_sql_actions(box iter, &query, ast)?, query)
}

// pub fn create_cf(cf_name: &str) -> Result<Response<Body>, Error> {
//     rocks::create_cf(cf_name)?;

//     Ok(WriteReply::<&str> {
//         result: ResultEmbeddedReply::ok(),
//         id: None,
//     }
//     .into())
// }

pub fn get<'a>(cf: &'a str, id: &'a str, query: Option<Query>) -> Result<Response<Body>, Error> {
    let iter = rocks::get(&cf, &id)?;
    new_read_ok_iter(after_read_actions(box iter, &query)?)
}

pub fn get_all_dbs() -> Result<Response<Body>, Error> {
    let res = rocks::get_all_dbs()?;

    let v = serde_json::to_string(&res).map_err(Error::SerdeError)?;

    new_read_ok(v.as_bytes())
}

fn after_read_sql_actions(iter: BoxedSledgeIter,
                          query: &Option<Query>,
                          ast: Vec<Statement>)
                          -> Result<BoxedSledgeIter, Error> {
    let ch = get_channel(&query)?;
    let iter = with_channel(iter, ch, &query);

    Ok(Filters::new_sql(ast).apply(iter))
}

fn after_read_actions(iter: BoxedSledgeIter,
                      query: &Option<Query>)
                      -> Result<BoxedSledgeIter, Error> {
    let ch = get_channel(&query)?;
    let iter = with_channel(iter, ch, &query);

    let mods = query.as_ref().map(|q| Filters::new(q));
    let iter2 = match mods {
        Some(mut mods) => mods.apply(iter),
        None => iter,
    };

    Ok(iter2)
}

fn get_channel(query: &Option<Query>) -> Result<Option<Channel>, Error> {
    if let Some(channel_id) = query.as_ref().and_then(|q| q.channel.as_ref()) {
        let res = rocks::get("_channel", &channel_id)?
            .next()
            .ok_or_else(|| Error::ChannelNotFound(channel_id.to_string()))?;
        let c = Channel::new_vec(res.value)?;
        return Ok(Some(c));
    }

    Ok(None)
}

fn get_id(query: &Option<Query>,
          path_id: Option<&str>,
          req: Option<&Bytes>)
          -> Result<String, Error> {
    if let Some(q) = query {
        if let (Some(id), Some(req)) = (q.field_path.as_ref(), req) {
            let j: Value = serde_json::from_slice(req.as_ref()).map_err(Error::SerdeError)?;
            let val: &Value = json_nested_value(id, &j);
            return Ok(val.as_str()
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

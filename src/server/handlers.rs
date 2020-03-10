use bytes::Bytes;
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
use crate::components::iterator::{SledgeIterator, with_channel, with_channel_for_single_value};
use crate::components::rocks;
use crate::components::sql;
use crate::server::filters::Filters;
use crate::server::query::Query;
use crate::server::reply::Reply;
use crate::server::responses::{get_iterating_response, new_read_ok, new_read_ok_iter};

pub type BytesResult = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;
pub type BytesResultStream = Box<dyn Stream<Item=BytesResult> + Send + Sync>;
pub type BytesResultIterator = dyn Iterator<Item=BytesResult> + Send + Sync;


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

pub async fn since_prefix(query: Option<Query>, id: &str, cf_name: &str)
                          -> Result<Response<Body>, Error> {
    let iter = rocks::range_prefix(id, cf_name)?;


    get_iterating_response(after_read_actions(iter, &query)?, query)
}

pub async fn all(query: Option<Query>, cf_name: &str)
                 -> Result<Response<Body>, Error> {
    let iter = rocks::range_all(&query, None, cf_name)?;

    get_iterating_response(
        after_read_actions(iter, &query)?, query)
}

pub async fn all_reverse(query: Option<Query>, cf_name: &str)
                         -> Result<Response<Body>, Error> {
    let iter = rocks::range_all_reverse(cf_name)?;
    get_iterating_response(after_read_actions(iter, &query)?, query)
}

pub async fn since(query: Option<Query>, id: Option<&str>, cf_name: &str)
                   -> Result<Response<Body>, Error> {
    let id = get_id(&query, id, None)?;
    let iter = rocks::range(&query, id.as_ref(), cf_name)?;
    get_iterating_response(after_read_actions(iter, &query)?, query)
}

pub async fn put<'a>(cf: &str, query: &'a Option<Query>, path_id: Option<&str>, req: Body)
                     -> Result<Response<Body>, Error> {
    let value = hyper::body::to_bytes(req)
        .await
        .map_err(Error::BodyParsingError)?;
    let id = get_id(&query, path_id, Some(&value))?;

    let ch = get_channel(&query)?;
    let value = with_channel_for_single_value(value, ch, query);
    rocks::put(cf, &id, value)?;

    let res = Reply::ok(None);
    Ok(res.into())
}

pub async fn sql(query: Option<Query>, req: Body) -> Result<Response<Body>, Error> {
    let value = hyper::body::to_bytes(req)
        .await
        .map_err(Error::BodyParsingError)?;
    let sql = std::str::from_utf8(value.as_ref())
        .map_err(|err| Error::Utf8Error(err.to_string()))?;

    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql.to_string())
        .map_err(Error::SqlError)?;

    let from = sql::utils::get_from(&ast).ok_or(Error::CFNotFound("".to_string()))?;

    let iter = rocks::range_all(&None, None, from.as_str())?;

    get_iterating_response(after_read_sql_actions(iter, &query, ast)?, query)
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
    let ch = get_channel(&query)?;

    new_read_ok_iter(after_read_actions(iter, &query)?)
}

pub fn get_all_dbs() -> Result<Response<Body>, Error> {
    let res = rocks::get_all_dbs()?;

    let v = serde_json::to_string(&res).map_err(Error::SerdeError)?;

    new_read_ok(v.as_bytes(), None)
}


fn after_read_sql_actions(iter: SledgeIterator, query: &Option<Query>, ast: Vec<Statement>) -> Result<SledgeIterator, Error> {
    let ch = get_channel(&query)?;
    let iter = with_channel(iter, ch, &query);

    let mods = Filters::new_sql(ast).apply(iter);

    Ok(mods)
}


fn after_read_actions(iter: SledgeIterator, query: &Option<Query>) -> Result<SledgeIterator, Error> {
    let ch = get_channel(&query)?;
    let iter = with_channel(iter, ch, &query);

    let mods = query.as_ref()
        .map(|q| Filters::new(q));
    let iter2 = match mods {
        Some(mut mods) => mods.apply(iter),
        None => iter,
    };

    Ok(iter2)
}

fn get_channel(query: &Option<Query>) -> Result<Option<Channel>, Error> {
    if let Some(channel_id) = query.as_ref().and_then(|q| q.channel.as_ref()) {
        let res = rocks::get("_channel", &channel_id)?.next()
            .ok_or(Error::ChannelNotFound(channel_id.to_string()))?;
        let c = Channel::new_vec(res.value)?;
        return Ok(Some(c));
    }

    return Ok(None);
}

fn get_id(query: &Option<Query>, path_id: Option<&str>, req: Option<&Bytes>)
          -> Result<String, Error> {
    if let Some(q) = query {
        if let (Some(id), Some(req)) = (q.field_path.as_ref(), req) {
            let j: Value = serde_json::from_slice(req.as_ref()).map_err(Error::SerdeError)?;
            let val: &Value = json_nested_value(id, &j);
            return Ok(val.as_str().ok_or(Error::IdNotFoundInJSON(id.clone()))?.to_string());
        }
    }

    let id = path_id.ok_or(Error::NoIdFoundOnRequest)?;
    match id {
        "_auto" => Ok(Uuid::new_v4().to_string()),
        _ => Ok(id.to_string()),
    }
}

pub fn json_nested_value<'a>(k: &str, v: &'a Value) -> &'a Value {
    k.split('.').fold(v, move |acc, x| &acc[x])
}
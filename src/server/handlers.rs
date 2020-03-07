use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use serde_json::Value;
use uuid::Uuid;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::errors::Error;
use crate::components::rocks;
use crate::components::rocks::SledgeIterator;
use crate::components::simple_pair::SimplePair;
use crate::server::query::Query;
use crate::server::reply::Reply;
use crate::server::responses::{get_iterating_response, new_read_ok, new_read_ok_iter};

pub type BytesResult = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;
pub type BytesResultStream = Box<dyn Stream<Item=BytesResult> + Send + Sync>;
pub type BytesResultIterator = dyn Iterator<Item=BytesResult> + Send + Sync;

struct IndexedValue {
    key: String,
    value: String,
}

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
    get_iterating_response(iter, query)
}

pub async fn all(query: Option<Query>, cf_name: &str)
                 -> Result<Response<Body>, Error> {
    let iter = rocks::range_all(&query, None, cf_name)?;
    get_iterating_response(iter, query)
}

pub async fn all_reverse(query: Option<Query>, cf_name: &str)
                         -> Result<Response<Body>, Error> {
    let iter = rocks::range_all_reverse(cf_name)?;
    get_iterating_response(iter, query)
}

pub async fn since(query: Option<Query>, id: Option<&str>, cf_name: &str)
                   -> Result<Response<Body>, Error> {
    let id = get_id(&query, id, None)?;
    let iter = rocks::range(&query, id.as_ref(), cf_name)?;
    get_iterating_response(iter, query)
}

pub async fn put<'a>(cf: &str, query: &'a Option<Query>, path_id: Option<&str>, req: Body)
                     -> Result<Response<Body>, Error> {
    let value = hyper::body::to_bytes(req)
        .await
        .map_err(Error::BodyParsingError)?;
    let id = get_id(&query, path_id, Some(&value))?;

    // let data = match channel {
    //     Some(ch) => parse_and_modify_u8(value.as_ref(), ch).map(Bytes::from)?,
    //     None => value,
    // };

    rocks::put(cf, &id, value)?;

    let res = Reply::ok(None);
    Ok(res.into())
}

// pub fn create_cf(cf_name: &str) -> Result<Response<Body>, Error> {
//     rocks::create_cf(cf_name)?;

//     Ok(WriteReply::<&str> {
//         result: ResultEmbeddedReply::ok(),
//         id: None,
//     }
//     .into())
// }

pub fn get<'a>(cf: &'a str, id: &'a str) -> Result<Response<Body>, Error> {
    let value = rocks::get(&cf, &id)?;

    // let data = match channel {
    //     Some(ch) => parse_and_modify_u8(value.as_ref(), &ch).map(Bytes::from)?,
    //     None => Bytes::from(value),
    // };

    // new_read_ok(data.as_ref(), Some(id))
    new_read_ok_iter(value)
}

pub fn get_all_dbs() -> Result<Response<Body>, Error> {
    let res = rocks::get_all_dbs()?;

    let v = serde_json::to_string(&res).map_err(Error::SerdeError)?;

    new_read_ok(v.as_bytes(), None)
}

// fn apply_ch(iter: SledgeIterator, ch: Channel) -> SledgeIterator {
//     box iter.flat_map(move |sp: SimplePair| parse_and_modify_u8(sp.v.as_slice(), &ch))
// }

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
                let res = rocks::get("_channel", &channel_id)?.next()
                    .ok_or(Error::ChannelNotFound(channel_id.to_string()))?;
                let c = Channel::new_vec(res.v)?;
                Ok(Some(c))
            }
            None => Ok(None),
        },
    }
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

fn json_nested_value<'a>(k: &str, v: &'a Value) -> &'a Value {
    k.split('.').fold(v, move |acc, x| &acc[x])
}
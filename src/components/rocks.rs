use std::convert::Infallible;
use std::env;
use std::iter::{Filter, Take};

use bytes::Bytes;
use futures::Stream;
use http::Response;
use hyper::Body;
use rocksdb::{DBIterator, IteratorMode, Options};
use serde_json::Value;
use uuid::Uuid;

use lazy_static::lazy_static;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::storage::{Error, IterMod, put_error};
use crate::server::query::Query;
use crate::server::responses::{new_range_result, new_read_error};

lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap();
        let db = new_storage(maybe_path);

        db
    };
}

pub async fn range(maybe_query: Option<Query>, maybe_path_id: Option<&str>, cf_name: &str, maybe_channel: Option<Channel>) -> Result<Response<Body>, Infallible> {
    let maybe_id = get_id(&maybe_query, maybe_path_id, None);
    let direction = get_range_direction(&maybe_query);

    let id: String;
    let mode = if maybe_id.is_some() {
        id = maybe_id.unwrap();
        IteratorMode::From(id.as_bytes(), direction)
    } else {
        match direction {
            rocksdb::Direction::Forward => IteratorMode::Start,
            rocksdb::Direction::Reverse => IteratorMode::End,
        }
    };

    let cf = match DB.cf_handle(cf_name) {
        Some(cf) => cf,
        None => return Ok(new_read_error("cf not found", None, Some(cf_name.into()))),
    };

    let source_iter: DBIterator = match DB.iterator_cf(cf, mode) {
        Ok(i) => i,
        Err(err) => return Ok(new_read_error(err, None, Some(cf_name.into())))
    };

    let new_iter: Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)> + Send + Sync> = box source_iter;
    let iter = match get_itermods(&maybe_query) {
        Some(iterators) => {
            iterators.into_iter().fold(new_iter, |acc, m| {
                match m {
                    IterMod::Limit(n) => box Iterator::take(acc, n),
                    IterMod::Skip(n) => box Iterator::skip(acc, n),
                    IterMod::UntilKey(id) => box Iterator::take_while(acc,move |x|x.0.to_vec() != Vec::from(id.clone())),       //TODO Fix this... please...
                    _ => acc
                }
            })
        }
        _ => new_iter
    };

    let stream: Box<dyn Stream<Item=Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync> =
        match maybe_channel {
            Some(ch) => {
                let db_iter = iter
                    .flat_map(move |tuple| parse_and_modify_u8(tuple.1.as_ref(), &ch).ok()
                        .and_then(|x| Some((tuple.0, x))))
                    .flat_map(|tuple| new_range_result(tuple.0.as_ref(), &tuple.1))
                    .map(|v| Ok(Bytes::from(v)));

                box futures::stream::iter(db_iter)
            }
            None => {
                let db_iter = iter
                    .flat_map(|tuple| new_range_result(tuple.0.as_ref(), tuple.1.as_ref()))
                    .map(|v| Ok(Bytes::from(v)));

                box futures::stream::iter(db_iter)
            }
        };

    let response = http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream))
        .unwrap();

    Ok(response)
}

pub fn new_storage(path: String) -> rocksdb::DB {
    let mut opts = Options::default();
    opts.create_if_missing(true);

    match rocksdb::DB::list_cf(&opts, path.clone()) {
        Ok(cfs) => {
            match rocksdb::DB::open_cf(&opts, path, cfs) {
                Ok(db) => db,
                Err(err) => panic!(err),
            }
        }
        Err(e) => {
            log::warn!("{}", e.to_string());
            rocksdb::DB::open(&opts, path).unwrap()
        }
    }
}


pub fn get(keyspace: &String, k: &String) -> Result<Vec<u8>, Error> {
    let cf = DB.cf_handle(&keyspace);

    let res = match cf {
        Some(cf) => DB.get_cf(cf, k.clone()),
        None => DB.get(k.clone()),
    }.or_else(|err| Err(Error::Get(err.to_string())))?;

    match res {
        Some(v) => Ok(v),
        None => Err(Error::NotFound(k.clone())),
    }
}

pub(crate) fn put(cf_name: &str, k: &String, v: Bytes) -> Result<(), Error> {
    let cf = DB.cf_handle(cf_name).ok_or(Error::CannotRetrieveCF(cf_name.to_string()))?;
    DB.put_cf(cf, k, v).or_else(put_error_with_rocks_err)
}

fn put_error_with_rocks_err(cause: rocksdb::Error) -> Result<(), Error> {
    put_error(cause.to_string())
}

fn get_itermods(maybe_query: &Option<Query>) -> Option<Vec<IterMod>> {
    if maybe_query.is_none() {
        return None;
    }

    let query = maybe_query.clone().unwrap();

    let mut itermods = Vec::new();
    if query.skip.is_some() {
        itermods.push(IterMod::Skip(query.skip.unwrap()))
    }
    if query.limit.is_some() {
        itermods.push(IterMod::Limit(query.limit.unwrap()))
    }
    if query.until_key.is_some() {
        itermods.push(IterMod::UntilKey(query.until_key.unwrap()))
    }

    Some(itermods)
}

fn get_range_direction(maybe_query: &Option<Query>) -> rocksdb::Direction {
    match &maybe_query {
        Some(query) => match query.direction_reverse {
            Some(is_reverse) => match is_reverse {
                true => rocksdb::Direction::Reverse,
                false => rocksdb::Direction::Forward,
            },
            _ => rocksdb::Direction::Forward,
        }
        _ => rocksdb::Direction::Forward
    }
}

pub fn get_id(maybe_query: &Option<Query>,
              maybe_path_id: Option<&str>,
              maybe_req: Option<&Bytes>) -> Option<String>
{
    match maybe_query {
        Some(q) => match (q.id.as_ref(), maybe_req) {
            (Some(id), Some(req)) => {
                let j: Value = serde_json::from_slice(req.as_ref()).ok()?;
                return Some(j[id].as_str()?.to_string());
            }
            _ => (),
        }
        _ => (),
    }

    match maybe_path_id {
        Some(path_id) => match path_id {
            "_auto" => Some(Uuid::new_v4().to_string()),
            _other => Some(path_id.to_string()),
        }
        None => None,
    }
}


#[test]
fn test_get_id() {
    let empty_input = None;
    let s = r#"{"my_key":"my_value"}"#;
    let json = Bytes::from(s);


    assert_eq!(get_id(&Some(Query {
        id: Some("my_key".to_string()),
        end: None,
        limit: None,
        until_key: None,
        skip: None,
        direction_reverse: None,
        channel: None,
    }), None, empty_input), None);

    assert_eq!(get_id(&Some(Query {
        id: Some("my_key".to_string()),
        end: None,
        limit: None,
        until_key: None,
        skip: None,
        direction_reverse: None,
        channel: None,
    }), Some("hello"),
                      Some(&json)), Some("my_value".to_string()));

    assert_eq!(get_id(&None, Some("my_key2"), empty_input), Some("my_key2".to_string()));
    assert_eq!(get_id(&None, Some("my_key2"), empty_input), Some("my_key2".to_string()));
    assert!(get_id(&None, Some("_auto"), empty_input).is_some());
    assert_eq!(get_id(&None, None, empty_input), None);
}
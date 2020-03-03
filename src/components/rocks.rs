use std::env;

use bytes::Bytes;
use rocksdb::{ColumnFamily, IteratorMode, Options};
use uuid::Uuid;

use lazy_static::lazy_static;
use serde_json::Value;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::kv::KV;
use crate::components::storage::{create_keyspace_error, Error, IterMod, put_error};
use crate::server::query::Query;
use futures::{Stream, future};
use http::Response;
use hyper::Body;
use std::convert::Infallible;

lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap();
        let mut db = new_storage(maybe_path);

        db
    };
}



pub fn range(maybe_query: Option<Query>, maybe_path_id: Option<&str>, cf_name: &str) -> Result<Response<Body>, Infallible> {
    let cf = match DB.cf_handle(cf_name) {
        Some(cf) => cf,
        None => return Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/text",
            )
            .body(Body::from("no cf found!"))
            .unwrap()),
    };

    let maybe_id = get_id(&maybe_query, maybe_path_id, None);
    let direction = get_range_direction(&maybe_query);

    let mut id: String;
    let mode = if maybe_id.is_some() {
        id = maybe_id.unwrap();
        IteratorMode::From(id.as_bytes(), direction)
    } else {
        match direction {
            rocksdb::Direction::Forward => IteratorMode::Start,
            rocksdb::Direction::Reverse => IteratorMode::End,
        }
    };

    let itermods = get_itermods(&maybe_query);

    let db_iter = match DB.iterator_cf(cf, mode) {
        Ok(i) => i,
        Err(err) => return Ok(http::Response::builder()
            .header(
                "Content-Type",
                "application/text",
            )
            .body(Body::from(err.to_string()))
            .unwrap())
    }
        .map(|tuple| tuple.1.to_vec())
        .map(|v: Vec<u8>| Ok(Bytes::from(v)));

    let stream: Box<dyn Stream<Item=Result<Bytes, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync> = box futures::stream::iter(db_iter);
    let response = http::Response::builder()
        .header(
            "Content-Type",
            "application/octet-stream",
        )
        .body(Body::from(stream))
        .unwrap();

    // return future::ok(response);
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


pub fn get(keyspace: String, k: String) -> Result<String, Error> {
    let cf = DB.cf_handle(&keyspace);

    let res = match cf {
        Some(cf) => DB.get_cf(cf, k.clone()),
        None => DB.get(k.clone()),
    }.or_else(|err| Err(Error::Get(err.to_string())))?;

    match res {
        Some(v) => match String::from_utf8(v) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::ParseFromUtf8(e)),
        },
        None => Err(Error::NotFound(k)),
    }
}


pub(crate) fn put(cf_name: String, k: String, v: Bytes) -> Result<(), Error> {
    let cf = DB.cf_handle(&cf_name).ok_or(Error::CannotRetrieveCF(cf_name))?;
    DB.put_cf(cf, k, v).or_else(put_error_with_rocks_err)
    // match (maybe_cf, self.create_cf_if_missing) {
    //     (Some(cf), _) => DB.put_cf(cf, k, v).or_else(put_error_with_rocks_err),
    //     (None, true) => {
    //         DB.create_cf(cf_name.clone(), &rocksdb::Options::default())
    //             .or_else(|err| keyspace_error_with_rocks_err(&cf_name.clone(), err))?;
    //
    //         let cf = DB.cf_handle(&cf_name)
    //             .ok_or(Error::CannotCreateKeyspace(cf_name, "unknown error. Keyspace not created".into()))?;
    //
    //         DB.put_cf(cf, k, v)
    //             .or_else(put_error_with_rocks_err)
    //     }
    //     (None, false) => put_error("cf not found and 'create if missing' is false".into()),
    // }
}


// pub fn process_kvs_with_ch(i: StorageIter, maybe_channel: Option<Channel>) -> Vec<u8> {
//     let ch: Channel;
//     let ch_name: String;
//
//     let channelized_iter = if maybe_channel.is_some() {
//         ch = maybe_channel.unwrap();
//         ch_name = ch.name.clone();
//
//         box i.map(|x| {
//             parse_and_modify_u8(x.value.as_ref(), &ch)
//                 .unwrap_or_else(|err| {
//                     log::warn!("error trying to pass value through channel '{}': {}", ch_name, err.to_string());
//                     x.value
//                 })
//         }) as VecIter
//     } else {
//         box i.map(|kv| kv.value) as VecIter
//     };
//
//     channelized_iter.flat_map(move |mut x| {
//         x.push('\n' as u8);
//         x
//     }).collect::<Vec<u8>>()//TODO Avoid this collect? Body::from has this `impl From<Box<dyn Stream<Item = Result<Bytes, Box<dyn StdError + Send + Sync>>> + Send + Sync>>`
// }

// pub fn handle_put(maybe_path_id: Option<&str>,
//                   maybe_query: Option<Query>,
//                   keyspace: String,
//                   req: Bytes)
//                   -> Result<impl Reply, Infallible>
// {
//     let v1_locked = db.lock().await;
//     let mut v1 = v1_locked;
//
//     let maybe_channel = match get_channel(&maybe_query).and_then(|x| x.channel) {
//         Ok(res) => res,
//         Err(err) => return errors::new_write_string(err.to_string(), Some("_channel".to_string())),
//     };
//
//     let data = match maybe_channel {
//         Some(c) => match parse_and_modify_u8(req.as_ref(), &c) {
//             Ok(v) => Bytes::from(v),
//             Err(err) => return errors::new_write(err.to_string().as_ref(), Some("_channel".to_string())),
//         },
//         None => req,
//     };
//
//     let id = match get_id(&maybe_query, maybe_path_id, &data) {
//         None => return errors::new_write("no id found for your document", Some(keyspace)),
//         Some(s) => s,
//     };
//
//     match v1.put(Some(keyspace.clone()), id.clone(), data) {
//         Ok(_) => responses::new_write(Some(id), Some(keyspace)),
//         Err(err) => errors::new_write(&err.to_string(), Some(keyspace)),
//     }
// }
//
// pub fn handle_get(keyspace: String, id: String, req: Option<Query>) -> future::Ready<Result<Response<Body>, hyper::Error>>
// {
//     let maybe_channel = match get_channel(&req) {
//         Ok(res) => res,
//         Err(err) => panic!(err),
//     };
//
//     let value: String = match get(keyspace.clone(), id) {
//         Ok(v) => v,
//         Err(err) => panic!(err),
//     };
//
//     let data = match maybe_channel {
//         Some(c) => match parse_and_modify_u8(value.as_ref(), &c) {
//             Ok(v) => Bytes::from(v),
//             Err(err) => panic!(err),
//         },
//         None => Bytes::from(value),
//     };
//
//     let maybe_value = serde_json::from_slice::<Value>(data.as_ref());
//     match maybe_value {
//         Ok(value) => future::ok(http::Response::builder()
//             .header(
//                 "Content-Type",
//                 "application/json",
//             )
//             .body(Body::from(value))
//             .unwrap()),
//         Err(err) => panic!(err),
//     }
// }


fn put_error_with_rocks_err(cause: rocksdb::Error) -> Result<(), Error> {
    put_error(cause.to_string())
}

fn keyspace_error_with_rocks_err(name: &str, cause: rocksdb::Error) -> Result<(), Error> {
    create_keyspace_error(name.into(), cause.to_string())
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
            other => Some(path_id.to_string()),
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
use std::env;

use bytes::Bytes;
use http::Response;
use hyper::Body;
use rocksdb::{DBIterator, IteratorMode, Options};

use lazy_static::lazy_static;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::storage::{Error, IterMod};
use crate::server::query::Query;
use crate::server::responses::{new_range_result, new_read_error};

lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap();
        let db = new_storage(maybe_path);

        db
    };
}

pub type StreamItem = Result<Bytes, Box<dyn std::error::Error + Send + Sync>>;
type ThreadByteIter = Box<dyn Iterator<Item=StreamItem> + Send + Sync>;
type RocksValue = (Box<[u8]>, Box<[u8]>);
type RocksIter = Box<dyn Iterator<Item=RocksValue> + Send + Sync>;

pub fn range(maybe_query: Option<Query>, maybe_id: Option<String>, cf_name: &str, maybe_channel: Option<Channel>)
             -> Result<ThreadByteIter, Response<Body>> {
    let direction = get_range_direction(&maybe_query);

    let mode = match maybe_id {
        Some(ref id) => IteratorMode::From(id.as_bytes(), direction),
        None => match direction {
            rocksdb::Direction::Forward => IteratorMode::Start,
            rocksdb::Direction::Reverse => IteratorMode::End,
        }
    };

    let cf = match DB.cf_handle(cf_name) {
        Some(cf) => cf,
        None => return Err(new_read_error("cf not found", None, Some(cf_name.into()))),
    };

    let source_iter: DBIterator = match DB.iterator_cf(cf, mode) {
        Ok(i) => i,
        Err(err) => return Err(new_read_error(err, None, Some(cf_name.into())))
    };

    let new_iter: RocksIter = box source_iter;
    let iter = match get_itermods(&maybe_query) {
        Some(iterators) => {
            iterators.into_iter().fold(new_iter, |acc, m| {
                match m {
                    IterMod::Limit(n) => box Iterator::take(acc, n),
                    IterMod::Skip(n) => box Iterator::skip(acc, n),
                    IterMod::UntilKey(id) => box Iterator::
                    take_while(acc, move |x| x.0.to_vec() != Vec::from(id.clone())),       //TODO Fix this... please...
                }
            })
        }
        _ => new_iter
    };

    let thread_iter: ThreadByteIter = match maybe_channel {
        Some(ch) => box iter
            .flat_map(move |tuple| parse_and_modify_u8(tuple.1.as_ref(), &ch).ok()
                .map(|x| (tuple.0, x)))
            .flat_map(|tuple| new_range_result(tuple.0.as_ref(), &tuple.1))
            .map(|v| Ok(Bytes::from(v))),

        None => box iter
            .flat_map(|tuple| new_range_result(tuple.0.as_ref(), tuple.1.as_ref()))
            .map(|v| Ok(Bytes::from(v))),
    };

    Ok(thread_iter)
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


pub fn get<'a>(keyspace: &'a str, k: &'a str) -> Result<Vec<u8>, Error<'a>> {
    let cf = DB.cf_handle(&keyspace);

    let res = match cf {
        Some(cf) => DB.get_cf(cf, k.clone()),
        None => DB.get(k.clone()),
    }.or_else(|err| Err(Error::Get(err.to_string())))?;

    match res {
        Some(v) => Ok(v),
        None => Err(Error::NotFound(k)),
    }
}

pub(crate) fn put<'a>(cf_name: &str, k: &str, v: Bytes) -> Result<(), Error<'a>> {
    let cf = DB.cf_handle(cf_name).ok_or_else(|| Error::CannotRetrieveCF(cf_name.to_string()))?;
    DB.put_cf(cf, k, v).or_else(|err| Err(Error::Put(err.to_string())))
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

use std::env;

use bytes::Bytes;
use rocksdb::{DBIterator, IteratorMode, Options};
use rocksdb::DBRawIterator;

use lazy_static::lazy_static;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::errors::Error;
use crate::components::simple_pair::SimplePair;
use crate::server::query::Query;

lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string());
        let db = new_storage(maybe_path);

        db
    };
}

pub enum IterMod {
    Skip(usize),
    Limit(usize),
    UntilKey(String),
}


pub type SledgeIterator = Box<dyn Iterator<Item=SimplePair> + Send + Sync>;

type RocksValue = (Box<[u8]>, Box<[u8]>);
type RocksIter = Box<dyn Iterator<Item=RocksValue> + Send + Sync>;

pub struct RawIteratorWrapper<'a> {
    pub inner: DBRawIterator<'a>,
}

impl Iterator for RawIteratorWrapper<'_> {
    type Item = SimplePair;

    fn next<'b>(&mut self) -> Option<<Self as Iterator>::Item> {
        self.inner.next();
        if !self.inner.valid() {
            return None;
        }

        let k = self.inner.key()?;
        let v = self.inner.value()?;

        Some(SimplePair::new_u8(k, v))
    }
}

pub fn range_all(
    query: &Option<Query>,
    id: Option<String>,
    cf_name: &str,
) -> Result<SledgeIterator, Error> {
    let direction = get_range_direction(query);
    let mode = match direction {
        rocksdb::Direction::Forward => IteratorMode::Start,
        rocksdb::Direction::Reverse => IteratorMode::End,
    };

    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

    let sledge_iter: SledgeIterator = box source_iter.map(|i| SimplePair::new_boxed(i));

    Ok(sledge_iter)
}

pub fn range_all_reverse(cf_name: &str) -> Result<SledgeIterator, Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB.iterator_cf(cf, IteratorMode::End).map_err(Error::RocksDB)?;
    let sledge_iter: SledgeIterator = box source_iter.map(|i| SimplePair::new_boxed(i));

    Ok(sledge_iter)
}

pub fn range(
    query: &Option<Query>,
    id: &str,
    cf_name: &str,
) -> Result<SledgeIterator, Error> {
    let direction = get_range_direction(query);
    let mode = IteratorMode::From(id.as_bytes(), direction);

    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

    let sledge_iter: SledgeIterator = box source_iter.map(|i| SimplePair::new_boxed(i));

    Ok(sledge_iter)

    // let new_iter: RocksIter = box source_iter;
    // let iter = match get_itermods(&query) {
    //     None => new_iter,
    //     Some(iterators) => {
    //         iterators.into_iter().fold(new_iter, |acc, m| {
    //             match m {
    //                 IterMod::Limit(n) => box Iterator::take(acc, n),
    //                 IterMod::Skip(n) => box Iterator::skip(acc, n),
    //                 IterMod::UntilKey(id) => box Iterator::take_while(acc, move |x| {
    //                     x.0.to_vec() != Vec::from(id.clone())
    //                 }), //TODO Fix this...
    //             }
    //         })
    //     }
    // };

    // let thread_iter: RangeResultIterator = match channel {
    //     Some(ch) => box iter
    //         .flat_map(move |tuple| {
    //             parse_and_modify_u8(tuple.1.as_ref(), &ch)
    //                 .ok()
    //                 .map(|x| (tuple.0, x))
    //         })
    //         .flat_map(|tuple| RangeResult::new_maybe(tuple.0.as_ref(), &tuple.1)),

    //     None => {
    //         box iter.flat_map(|tuple| RangeResult::new_maybe(tuple.0.as_ref(), tuple.1.as_ref()))
    //     }
    // };

    // Ok(thread_iter)
}

pub fn range_prefix<'a>(id: &str, cf_name: &str) -> Result<SledgeIterator, Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let mut iter = DB.raw_iterator_cf(cf).map_err(Error::RocksDB)?;
    iter.seek(id);

    let ret_iter: SledgeIterator = box RawIteratorWrapper { inner: iter };

    Ok(ret_iter)
}

pub fn get<'a>(db: &'a str, id: &'a str) -> Result<SledgeIterator, Error> {
    let cf = DB
        .cf_handle(&db)
        .ok_or_else(|| Error::CFNotFound(db.to_string()))?;

    // let db_: rocksdb::DB;
    let res = DB
        .get_cf(cf, id)
        .map_err(Error::RocksDB)?
        .ok_or_else(|| Error::NotFound(id.to_string()))
        .map(|v| box vec![SimplePair::new_str_vec(id, v)]
            .into_iter())?;

    Ok(res)
}

pub fn put<'a>(cf_name: &str, k: &str, v: Bytes) -> Result<(), Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CannotRetrieveCF(cf_name.to_string()))?;
    DB.put_cf(cf, k, v)
        .or_else(|err| Err(Error::Put(err.to_string())))
}

// pub fn create_cf(cf_name: &str)-> Result<(),Error> {
//     DB.create_cf(cf_name, &rocksdb::Options::default())
//         .map_err(Error::RocksDB)
// }

pub fn get_all_dbs() -> Result<Vec<String>, Error> {
    rocksdb::DB::list_cf(
        &rocksdb::Options::default(),
        env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string()),
    )
        .map_err(Error::RocksDB)
}

pub fn new_storage(path: String) -> rocksdb::DB {
    let mut opts = Options::default();
    opts.create_if_missing(true);

    match rocksdb::DB::list_cf(&opts, path.clone()) {
        Ok(cfs) => match rocksdb::DB::open_cf(&opts, path, cfs) {
            Ok(db) => db,
            Err(err) => panic!(err),
        },
        Err(e) => {
            log::warn!("{}", e.to_string());
            rocksdb::DB::open(&opts, path).unwrap()
        }
    }
}

fn get_itermods(query: &Option<Query>) -> Option<Vec<IterMod>> {
    match query {
        None => None,
        Some(query) => {
            let mut itermods = Vec::new();
            if let Some(skip) = query.skip {
                itermods.push(IterMod::Skip(skip))
            }

            if let Some(limit) = query.limit {
                itermods.push(IterMod::Limit(limit))
            }

            if let Some(ref until_key) = query.until_key {
                itermods.push(IterMod::UntilKey(until_key.clone()))
            }

            if itermods.is_empty() {
                return None;
            }

            Some(itermods)
        }
    }
}

fn get_range_direction(query: &Option<Query>) -> rocksdb::Direction {
    if let Some(q) = query {
        if let Some(is_reverse) = q.direction_reverse {
            if is_reverse {
                return rocksdb::Direction::Reverse;
            };
        }
    };

    rocksdb::Direction::Forward
}

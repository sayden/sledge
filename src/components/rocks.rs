use std::env;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use bytes::Bytes;
use rocksdb::{DBIterator, IteratorMode, Options};

use lazy_static::lazy_static;

use crate::components::errors::Error;
use crate::components::iterator::RawIteratorWrapper;
use crate::components::simple_pair::SimplePair;
use crate::server::query::Query;

pub trait SledgeIterator = Iterator<Item = SimplePair> + Send + Sync;

//TODO Remove this static use and initialization of db
lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string());
        new_storage(maybe_path)
    };
}

pub fn range_all(
    query: &Option<Query>,
    _id: Option<String>,
    cf_name: &str,
) -> Result<impl SledgeIterator, Error> {
    let direction = get_range_direction(query);
    let mode = match direction {
        rocksdb::Direction::Forward => IteratorMode::Start,
        rocksdb::Direction::Reverse => IteratorMode::End,
    };

    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

    let sledge_iter = source_iter.map(SimplePair::new_boxed);

    Ok(sledge_iter)
}

pub fn range_all_reverse(cf_name: &str) -> Result<impl SledgeIterator, Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB
        .iterator_cf(cf, IteratorMode::End)
        .map_err(Error::RocksDB)?;
    let sledge_iter = source_iter.map(SimplePair::new_boxed);

    Ok(sledge_iter)
}

pub fn range(query: &Option<Query>, id: &str, cf_name: &str) -> Result<impl SledgeIterator, Error> {
    let direction = get_range_direction(query);
    let mode = IteratorMode::From(id.as_bytes(), direction);

    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter: DBIterator = DB.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

    let sledge_iter = source_iter.map(SimplePair::new_boxed);

    Ok(sledge_iter)
}

pub fn range_prefix(id: &str, cf_name: &str) -> Result<impl SledgeIterator, Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let mut iter = DB.raw_iterator_cf(cf).map_err(Error::RocksDB)?;
    iter.seek(id);

    let ret_iter = RawIteratorWrapper { inner: iter };

    Ok(ret_iter)
}

pub fn get_with_db(
    db: &RwLockReadGuard<rocksdb::DB>,
    cf: &str,
    id: &str,
) -> Result<impl SledgeIterator, Error> {
    let cf = db
        .cf_handle(&cf)
        .ok_or_else(|| Error::CFNotFound(cf.to_string()))?;

    let res = db
        .get_cf(cf, id)
        .map_err(Error::RocksDB)?
        .ok_or_else(|| Error::NotFound(id.to_string()))
        .map(|v| vec![SimplePair::new_str_vec(id, v)].into_iter())?;

    Ok(res)
}

pub fn get(cf: &str, id: &str) -> Result<impl SledgeIterator, Error> {
    let cf = DB
        .cf_handle(&cf)
        .ok_or_else(|| Error::CFNotFound(cf.to_string()))?;

    let res = DB
        .get_cf(cf, id)
        .map_err(Error::RocksDB)?
        .ok_or_else(|| Error::NotFound(id.to_string()))
        .map(|v| vec![SimplePair::new_str_vec(id, v)].into_iter())?;

    Ok(res)
}

pub fn put(cf_name: &str, k: &str, v: Bytes) -> Result<(), Error> {
    let cf = DB
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CannotRetrieveCF(cf_name.to_string()))?;

    let mut res: rocksdb::FlushOptions = rocksdb::FlushOptions::default();
    res.set_wait(true);
    DB.put_cf(cf, k, v)
        .and(DB.flush_opt(&res))
        .or_else(|err| Err(Error::Put(err.to_string())))
}

pub fn create_cf(db: Arc<RwLock<rocksdb::DB>>, cf: &str) -> Result<(), Error> {
    let mut inner = db.write().unwrap();
    inner
        .create_cf(cf, &rocksdb::Options::default())
        .map_err(|err| Error::CannotCreateDb(cf.to_string(), err.to_string()))?;
    log::debug!("column family '{}' created", cf);

    Ok(())
}

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

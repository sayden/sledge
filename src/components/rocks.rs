use std::env;
use std::sync::{Arc, RwLock};

use rocksdb::{DBIterator, Direction, IteratorMode, Options};

use lazy_static::lazy_static;

use crate::components::errors::Error;
use crate::components::raw_iterator::RawIteratorWrapper;
use crate::components::simple_pair::SimplePair;

pub trait SledgeIterator = Iterator<Item = SimplePair> + Send + Sync;

//TODO Remove this static use and initialization of db
lazy_static! {
    static ref DB: rocksdb::DB = {
        let maybe_path = env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string());
        new_storage(maybe_path)
    };
}

pub fn rangef<F>(
    db: Arc<RwLock<rocksdb::DB>>,
    is_reverse: bool,
    id: Option<&str>,
    cf_name: &str,
    f: F,
) -> Result<Vec<SimplePair>, Error>
where
    F: FnOnce(DBIterator) -> Vec<SimplePair>,
{
    let mode = get_range_mode(is_reverse, id);

    let db = db.read().unwrap();

    let cf = db
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let source_iter = db.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

    let sledge_iter = f(source_iter);

    Ok(sledge_iter)
}

pub fn range_prefix<F>(
    db: Arc<RwLock<rocksdb::DB>>,
    id: &str,
    cf_name: &str,
    f: F,
) -> Result<Vec<SimplePair>, Error>
where
    F: FnOnce(RawIteratorWrapper) -> Vec<SimplePair>,
{
    let db = db.read().unwrap();

    let cf = db
        .cf_handle(cf_name)
        .ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
    let mut iter = db.raw_iterator_cf(cf).map_err(Error::RocksDB)?;
    iter.seek(id);

    let ret_iter = RawIteratorWrapper { inner: iter };
    let res = f(ret_iter);
    Ok(res)
}

pub fn get<F>(
    db: Arc<RwLock<rocksdb::DB>>,
    cf: &str,
    id: &str,
    f: F,
) -> Result<Vec<SimplePair>, Error>
where
    F: FnOnce(SimplePair) -> Vec<SimplePair>,
{
    let db = db.read().unwrap();

    let cf = db
        .cf_handle(&cf)
        .ok_or_else(|| Error::CFNotFound(cf.to_string()))?;

    let res = db
        .get_cf(cf, id)
        .map_err(Error::RocksDB)?
        .ok_or_else(|| Error::NotFound(id.to_string()))
        .map(|v| SimplePair::new_str_vec(id, v))?;

    let result = f(res);

    Ok(result)
}

pub fn put(cf_name: &str, k: Vec<u8>, v: Vec<u8>) -> Result<(), Error> {
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

fn get_range_mode(is_reverse: bool, id: Option<&str>) -> rocksdb::IteratorMode {
    match id {
        Some(id) => {
            IteratorMode::From(
                id.as_ref(),
                if is_reverse {
                    Direction::Reverse
                } else {
                    Direction::Forward
                },
            )
        }
        None => {
            if is_reverse {
                IteratorMode::End
            } else {
                IteratorMode::Start
            }
        }
    }
}

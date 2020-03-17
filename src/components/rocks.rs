use std::{
  env,
  sync::{Arc, RwLock},
};

use rocksdb::{DBIterator, Direction, IteratorMode, Options, DB};

use crate::components::{errors::Error, simple_pair::SimplePair};

pub fn range<F>(db: Arc<RwLock<DB>>, is_reverse: bool, id: String, cf: &str, f: F) -> Result<Vec<SimplePair>, Error>
  where F: FnOnce(DBIterator) -> Vec<SimplePair>
{
  let mode = get_range_mode(is_reverse, Some(id.as_str()));
  let db = db.read().unwrap();
  let cf = db.cf_handle(cf).ok_or_else(|| Error::CFNotFound(cf.to_string()))?;

  let source_iter = db.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

  let vector = f(source_iter);

  Ok(vector)
}

pub fn range_prefix<F>(db: Arc<RwLock<DB>>, id: String, cf_name: &str, f: F) -> Result<Vec<SimplePair>, Error>
  where F: FnOnce(DBIterator) -> Vec<SimplePair>
{
  let db = db.read().unwrap();

  let cf = db.cf_handle(cf_name).ok_or_else(|| Error::CFNotFound(cf_name.to_string()))?;
  let iter = db.prefix_iterator_cf(cf, id).map_err(Error::RocksDB)?;

  let vector = f(iter);

  Ok(vector)
}

pub fn try_streaming<F, R>(db: Arc<RwLock<DB>>, f: F) -> Result<R, Error>
  where F: FnOnce(DBIterator) -> R
{
  let mode = get_range_mode(false, None);
  let db = db.read().unwrap();
  let cf = db.cf_handle("test_db").ok_or_else(|| Error::CannotRetrieveCF("test_db".to_string()))?;

  let source_iter = db.iterator_cf(cf, mode).map_err(Error::RocksDB)?;

  Ok(f(source_iter))
}

pub fn get<F>(db: Arc<RwLock<DB>>, cf: &str, id: &str, f: F) -> Result<Vec<SimplePair>, Error>
  where F: FnOnce(SimplePair) -> Vec<SimplePair>
{
  let db = db.read().unwrap();

  let cf = db.cf_handle(&cf).ok_or_else(|| Error::CFNotFound(cf.to_string()))?;

  let res = db.get_cf(cf, id)
              .map_err(Error::RocksDB)?
              .ok_or_else(|| Error::NotFound(id.to_string()))
              .map(|v| SimplePair::new_str_vec(id, v))?;

  let result = f(res);

  Ok(result)
}

pub fn put(db: Arc<RwLock<DB>>, cf_name: &str, k: Vec<u8>, v: Vec<u8>) -> Result<(), Error> {
  let db = db.write().unwrap();

  let cf = db.cf_handle(cf_name).ok_or_else(|| Error::CannotRetrieveCF(cf_name.to_string()))?;

  let mut res: rocksdb::FlushOptions = rocksdb::FlushOptions::default();
  res.set_wait(true);
  db.put_cf(cf, k, v).and(db.flush_opt(&res)).or_else(|err| Err(Error::Put(err.to_string())))
}

pub fn create_cf(db: Arc<RwLock<DB>>, cf: &str) -> Result<(), Error> {
  let mut inner = db.write().unwrap();
  inner.create_cf(cf, &rocksdb::Options::default())
       .map_err(|err| Error::CannotCreateDb(cf.to_string(), err.to_string()))?;
  log::debug!("column family '{}' created", cf);

  Ok(())
}

pub fn get_all_dbs() -> Result<Vec<String>, Error> {
  DB::list_cf(
        &rocksdb::Options::default(),
        env::var("FEEDB_PATH").unwrap_or_else(|_| "/tmp/storage".to_string()),
    )
    .map_err(Error::RocksDB)
}

pub fn new_storage(path: String) -> DB {
  let mut opts = Options::default();
  opts.create_if_missing(true);

  match DB::list_cf(&opts, path.clone()) {
    Ok(cfs) =>
      match DB::open_cf(&opts, path, cfs) {
        Ok(db) => db,
        Err(err) => panic!(err),
      },
    Err(e) => {
      log::warn!("{}", e.to_string());
      DB::open(&opts, path).unwrap()
    }
  }
}

fn get_range_mode(is_reverse: bool, id: Option<&str>) -> rocksdb::IteratorMode {
  match id {
    Some(id) => IteratorMode::From(id.as_bytes(), if is_reverse { Direction::Reverse } else { Direction::Forward }),
    None =>
      if is_reverse {
        IteratorMode::End
      } else {
        IteratorMode::Start
      },
  }
}

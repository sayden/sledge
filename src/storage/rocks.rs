use rocksdb::{DB, ColumnFamily, Options, IteratorMode};


use crate::components::storage::{Storage, Error, put_error, create_keyspace_error, IterMod, StorageIter};
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::env;
use bytes::Bytes;
use crate::server::requests::Query;

pub struct Rocks {
    db: rocksdb::DB,
    create_cf_if_missing: bool,
}

impl Rocks {
    pub fn new(path: String) -> impl Storage + Send + Sync {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let create_if_missing = env::var("ROCKSDB_CREATE_CF_IF_MISSING")
            .unwrap_or("true".to_string()) == "true";

        match DB::list_cf(&opts, path.clone()) {
            Ok(cfs) => {
                match DB::open_cf(&opts, path.clone(), cfs) {
                    Ok(db) => return Rocks { db, create_cf_if_missing: create_if_missing },
                    Err(err) => panic!(err),
                }
            }
            Err(e) => {
                log::warn!("{}", e.to_string());
                let db = DB::open(&opts, path.clone()).unwrap();
                return Rocks { db, create_cf_if_missing: create_if_missing };
            }
        };
    }
}


impl Storage for Rocks {
    fn get(&self, maybe_keyspace: Option<String>, k: String) -> Result<String, Error> {
        let cf = maybe_keyspace.and_then(|cf| self.db.cf_handle(&cf));

        let res = match cf {
            Some(cf) => self.db.get_cf(cf, k.clone()),
            None => self.db.get(k.clone()),
        }.or_else(|err| Err(Error::Get(err.to_string())))?;

        match res {
            Some(v) => match String::from_utf8(v) {
                Ok(v) => Ok(v),
                Err(e) => Err(Error::ParseFromUtf8(e)),
            },
            None => Err(Error::NotFound(k)),
        }
    }

    fn put(&mut self, maybe_keyspace: Option<String>, k: String, v: Bytes) -> Result<(), Error> {
        // Write to default column family
        if maybe_keyspace.is_none() {
            return self.db.put(k, v).or_else(put_error_with_rocks_err);
        }

        let cf_name = maybe_keyspace.unwrap();
        let maybe_cf = self.db.cf_handle(&cf_name);

        match (maybe_cf, self.create_cf_if_missing) {
            (Some(cf), _) => self.db.put_cf(cf, k, v).or_else(put_error_with_rocks_err),
            (None, true) => {
                self.db.create_cf(cf_name.clone(), &rocksdb::Options::default())
                    .or_else(|err| keyspace_error_with_rocks_err(&cf_name.clone(), err))?;

                let cf = self.db.cf_handle(&cf_name)
                    .ok_or(Error::CannotCreateKeyspace(cf_name, "unknown error. Keyspace not created".into()))?;

                self.db.put_cf(cf, k, v)
                    .or_else(put_error_with_rocks_err)
            }
            (None, false) => put_error("cf not found and 'create if missing' is false".into()),
        }
    }

    fn create_keyspace(&mut self, name: String) -> Result<(), Error> {
        let opt = rocksdb::Options::default();
        self.db.create_cf(&name, &opt)
            .or_else(|err| Err(Error::CannotCreateKeyspace(name, err.to_string())))
    }

    fn start(&self, maybe_keyspace: Option<String>) -> Result<StorageIter, Error> {
        self.rocks_iterator(maybe_keyspace, rocksdb::IteratorMode::Start)
    }

    fn end(&self, maybe_keyspace: Option<String>) -> Result<StorageIter, Error> {
        self.rocks_iterator(maybe_keyspace, rocksdb::IteratorMode::End)
    }

    fn range(&self, maybe_keyspace: Option<String>, query: Query) -> Result<StorageIter, Error> {
        let cf = self.get_column_family(maybe_keyspace)?;

        let mut itermods = Vec::new();
        if query.skip.is_some() {
            log::info!("found skip");
            itermods.push(IterMod::Skip(query.skip.unwrap()))
        }
        if query.limit.is_some() {
            log::info!("found limit");
            itermods.push(IterMod::Limit(query.limit.unwrap()))
        }
        if query.until_key.is_some() {
            log::info!("found until_key");
            itermods.push(IterMod::UntilKey(query.until_key.unwrap()))
        }

        let id = query.id.ok_or(Error::WrongQuery)?;

        //Check direction of the range query
        let mode = IteratorMode::From(id.as_bytes(), match query.direction_forward {
            Some(v) => match v {
                true => rocksdb::Direction::Forward,
                false => rocksdb::Direction::Reverse,
            },
            None => rocksdb::Direction::Forward,
        });

        //Perform query to db to get iterator
        let db_iter = match cf {
            Some(cf) => self.db.iterator_cf(cf, mode)
                .or_else(|err| Err(Error::Iterator(err.to_string()))),
            None => Ok(self.db.iterator(mode)),
        }?;

        let iter: Box<dyn Iterator<Item=(Box<[u8]>, Box<[u8]>)>> = Box::new(db_iter);

        let res = itermods.into_iter()
            .fold(iter, |acc, x| {
                match x {
                    IterMod::Limit(limit) => Box::new(Iterator::take(acc, limit)),
                    IterMod::Skip(skip) => Box::new(Iterator::skip(acc, skip)),
                    IterMod::UntilKey(key) => Box::new(
                        Iterator::take_while(acc, move |x|
                            x.0.to_vec() != key.as_bytes().to_vec())),
                }
            });

        Ok(Box::new(res.map(tuplebox_to_kv)))
    }


    fn since(&self, maybe_keyspace: Option<String>, k: String)
             -> Result<StorageIter, Error> {
        self.rocks_iterator(maybe_keyspace,
                            rocksdb::IteratorMode::From(k.as_bytes(), rocksdb::Direction::Forward))
    }

    fn since_until(&self, maybe_keyspace: Option<String>, k: String, k2: String)
                   -> Result<StorageIter, Error> {
        let res = self.rocks_iterator(
            maybe_keyspace,
            rocksdb::IteratorMode::From(k.as_bytes(), rocksdb::Direction::Forward))
            .and_then(|iter| Ok(iter.take_while(
                move |x| *x != k2)))?;

        Ok(Box::new(res))
    }

    fn reverse(&self, _maybe_keyspace: Option<String>, k: String)
               -> Result<StorageIter, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(db_iter.map(tuplebox_to_kv)))
    }

    fn reverse_until(&self, _maybe_keyspace: Option<String>, k1: String, k2: String)
                     -> Result<StorageIter, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k1.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(db_iter.map(tuplebox_to_kv)
            .take_while(move |kv| *kv != k2)))
    }

    fn stats(&self) -> Stats {
        unimplemented!()
    }
}

fn tuplebox_to_kv(z: (Box<[u8]>, Box<[u8]>)) -> KV {
    KV { key: z.0.into_vec(), value: z.1.into_vec() }
}

impl Rocks {
    fn get_column_family(&self, maybe_keyspace: Option<String>) -> Result<Option<&ColumnFamily>, Error> {
        match maybe_keyspace {
            Some(ks) => self.db.cf_handle(&ks)
                .ok_or(Error::CannotRetrieveCF(ks))
                .and_then(|cf| Ok(Some(cf))),
            None => Ok(None),
        }
    }

    fn rocks_iterator(&self, maybe_keyspace: Option<String>, mode: rocksdb::IteratorMode)
                      -> Result<StorageIter, Error> {
        let cf = self.get_column_family(maybe_keyspace)?;

        let db_iter = match cf {
            Some(cf) => self.db.iterator_cf(cf, mode)
                .or_else(|err| Err(Error::Iterator(err.to_string()))),
            None => Ok(self.db.iterator(mode)),
        }?;

        Ok(Box::new(db_iter.map(tuplebox_to_kv)))
    }
}

fn put_error_with_rocks_err(cause: rocksdb::Error) -> Result<(), Error> {
    put_error(cause.to_string())
}

fn keyspace_error_with_rocks_err(name: &str, cause: rocksdb::Error) -> Result<(), Error> {
    create_keyspace_error(name.into(), cause.to_string())
}
use rocksdb::{DB, DBIterator, ColumnFamily, Options};


use crate::conversions::vector::convert_vec_pairs;
use crate::components::storage::{Storage, Error};
use std::iter::FilterMap;
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::env;

pub struct Rocks {
    db: rocksdb::DB,
    create_cf_if_missing: bool,
}

impl Rocks {
    pub fn new(path: String) -> Box<dyn Storage + Send + Sync> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let create_if_missing = env::var("ROCKSDB_CREATE_CF_IF_MISSING")
            .unwrap_or("true".to_string()) == "true";

        match DB::list_cf(&opts, path.clone()) {
            Ok(cfs) => {
                match DB::open_cf(&opts, path.clone(), cfs) {
                    Ok(db) => return Box::new(Rocks { db, create_cf_if_missing: create_if_missing }),
                    Err(err) => panic!(err),
                }
            }
            Err(e) => {
                log::warn!("{}", e.to_string());
                let db = DB::open(&opts, path.clone()).unwrap();
                return Box::new(Rocks { db, create_cf_if_missing: create_if_missing });
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

    fn put(&mut self, maybe_keyspace: Option<String>, k: String, v: String) -> Result<(), Error> {
        if maybe_keyspace.is_none() {
            return self.db.put(k, v).or_else(|err| Err(Error::Put(err.to_string())));
        }

        let cf = maybe_keyspace.unwrap();
        let maybe_cf = self.db.cf_handle(&cf);

        if maybe_cf.is_none() && self.create_cf_if_missing {
            self.db.create_cf(cf.clone(), &rocksdb::Options::default())
                .or_else(|err| Err(Error::CannotCreateKeyspace(cf.clone(), err.to_string())))?;
            let cf_created = self.db.cf_handle(&cf)
                .ok_or(Error::CannotCreateKeyspace(cf, "unknown error".to_string()))?;

            self.db.put_cf(cf_created, k, v)
        } else {
            self.db.put(k, v)
        }
            .or_else(|err| Err(Error::Put(err.to_string())))
    }

    fn start<'a>(&'a self, maybe_keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        self.rocks_iterator(maybe_keyspace, rocksdb::IteratorMode::Start)
    }

    fn end<'a>(&'a self, maybe_keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        self.rocks_iterator(maybe_keyspace, rocksdb::IteratorMode::End)
    }

    fn since<'a>(&'a self, maybe_keyspace: Option<String>, k: String)
                 -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        self.rocks_iterator(maybe_keyspace,
                            rocksdb::IteratorMode::From(k.as_bytes(), rocksdb::Direction::Forward))
    }

    fn since_until<'a>(&'a self, maybe_keyspace: Option<String>, k: String, k2: String)
                       -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        let res = self.rocks_iterator(
            maybe_keyspace,
            rocksdb::IteratorMode::From(k.as_bytes(), rocksdb::Direction::Forward))
            .and_then(|iter| Ok(iter.take_while(
                move |x| x.key != k2)))?;

        Ok(Box::new(res))
    }

    fn reverse<'a>(&'a self, _maybe_keyspace: Option<String>, k: String)
                   -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }

    fn reverse_until<'a>(&'a self, _maybe_keyspace: Option<String>, k1: String, k2: String)
                         -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k1.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(Rocks::simple_iterator(db_iter).take_while(move |x| x.key != k2)))
    }

    fn create_keyspace(&mut self, name: String) -> Result<(), Error> {
        let opt = rocksdb::Options::default();
        self.db.create_cf(&name, &opt)
            .or_else(|err| Err(Error::CannotCreateKeyspace(name, err.to_string())))
    }

    fn stats(&self) -> Stats {
        unimplemented!()
    }
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

    fn simple_iterator(iter: DBIterator) -> FilterMap<DBIterator, fn((Box<[u8]>, Box<[u8]>)) -> Option<KV>> {
        iter.filter_map(Rocks::tuplebox_to_maybe_kv)
    }

    fn tuplebox_to_maybe_kv(z: (Box<[u8]>, Box<[u8]>)) -> Option<KV> {
        let (x, y) = (z.0, z.1);
        return match convert_vec_pairs(x.into_vec(), y.into_vec()) {
            Err(e) => print_err_and_none!(e),
            Ok(pair) => Some(pair),
        };
    }

    fn rocks_iterator<'a>(&'a self, maybe_keyspace: Option<String>, mode: rocksdb::IteratorMode)
                          -> Result<Box<dyn Iterator<Item=KV> + 'a>, Error> {
        let cf = self.get_column_family(maybe_keyspace)?;

        let db_iter = match cf {
            Some(cf) => self.db.iterator_cf(cf, mode)
                .or_else(|err| Err(Error::Iterator(err.to_string()))),
            None => Ok(self.db.iterator(mode)),
        }?;

        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }
}
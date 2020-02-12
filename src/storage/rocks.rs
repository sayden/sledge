use rocksdb::{DB, DBIterator};
use anyhow::Error;

use crate::conversions::vector::convert_vec_pairs;
use crate::components::storage::{Storage, SledgeIterator};
use std::iter::FilterMap;
use crate::components::kv::KV;

pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub fn new(path: String) -> Box<dyn Storage> {
        let db = DB::open_default(path).unwrap();
        Box::new(Rocks { db })
    }
}


impl Storage for Rocks {
    fn get(&self, s: String) -> Result<Option<String>, Error> {
        let result = self.db.get(s)?;

        let b = result.and_then(|r| match String::from_utf8(r) {
            Ok(v) => Some(v),
            Err(e) => print_err_and_none!(e),
        });

        Ok(b)
    }

    fn put(&mut self, k: String, v: String) -> Result<(), Error> {
        self.db.put(k, v)
            .or_else(|x| bail!(x))
    }

    fn start(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::Start);
        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }

    fn end(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::End);
        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }

    fn since(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }

    fn since_until(&self, k: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        Ok(Box::new(Rocks::simple_iterator(db_iter).take_while(move |x| x.key != k2)))
    }

    fn reverse(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(Rocks::simple_iterator(db_iter)))
    }

    fn reverse_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k1.as_bytes(),
                                                                   rocksdb::Direction::Reverse));
        Ok(Box::new(Rocks::simple_iterator(db_iter).take_while(move |x| x.key != k2)))
    }
}

impl Rocks {
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
}
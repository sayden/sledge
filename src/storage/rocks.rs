use rocksdb::{DB, DBIterator};
use anyhow::Error;

use crate::conversions::vector::convert_vec_pairs;
use crate::components::storage::{Storage, SledgeIterator, Options};
use crate::components::storage::{Bound, KV};
use crate::components::storage::Bound::{Limit, Key, KeyEqualsValue};

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
    fn get(&self, s: &str) -> Result<Option<String>, Error> {
        let result = self.db.get(s)?;

        let b = result.and_then(|r| match String::from_utf8(r) {
            Ok(v) => Some(v),
            Err(e) => print_err_and_none!(e),
        });

        Ok(b)
    }

    fn put(&self, k: &str, v: &str) -> Result<(), Error> {
        self.db.put(k, v)
            .or_else(|x| bail!(x))
    }

    fn since(&self, k: &str) -> Result<Box<SledgeIterator>, Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        let converted_to_string = db_iter
            .filter_map(|(x, y)| {
                return match convert_vec_pairs(x.into_vec(), y.into_vec()) {
                    Err(e) => print_err_and_none!(e),
                    Ok(pair) => Some(pair),
                };
            });

        Ok(Box::new(converted_to_string))
    }

    fn since_until(&self, k: &str, k2: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse(&self, k: &str) -> Result<Box<SledgeIterator>, Error> {
//        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
//                                                                   rocksdb::Direction::Reverse));
        unimplemented!()
    }

    fn reverse_until(&self, k: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}

impl Rocks {
//
//    fn iter_to_sledge_iterator(iter: DBIterator)-> Box<SledgeIterator> {
//        Box::new(iter.filter_map(|(x, y)| {
//            return match convert_vec_pairs(x.into_vec(), y.into_vec()) {
//                Err(e) => print_err_and_none!(e),
//                Ok(pair) => Some(pair),
//            };
//        }))
//    }

//    fn new_bound(b: Bound) -> (){
//        match b {
//            Limit(n: u32) => (),
//            Key(s: String) => (),
//            KeyEqualsValue(kv: KV) => (),
//            Infinite => (),
//        }
//    }
}
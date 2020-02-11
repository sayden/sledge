use anyhow::Error;
use sled::IVec;
use crate::conversions::vector::convert_vec_pairs_u8;
use crate::components::storage::{Storage, Options, SledgeIterator, KV};

pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn parse_potential_value(i: &Option<IVec>) -> Result<Option<String>, Error> {
        let value = i.as_ref().ok_or(anyhow::anyhow!("value not found"))?;
        return match std::str::from_utf8(value.as_ref()) {
            Ok(x) => Ok(Some(x.to_string())),
            Err(e) => bail!(e),
        };
    }
}


impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, Error> {
//        let db_result = self.db.get(s).or_else(|e| bail!(e)).unwrap();
        let db_result = self.db.get(s)?;
        let result = Sled::parse_potential_value(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<(), Error> {
        self.db.insert(k, v)
            .and_then(|_| Ok(()))
            .or_else(|x| bail!(x))
    }

    fn since(&self, k: &str) -> Result<Box<SledgeIterator>, Error> {
        let ranged_result = self.db.range(k..);

        let iter = ranged_result
            .filter_map(|item| {
                let i = item.or_else(|e| bail!(e)).unwrap();
                match convert_vec_pairs_u8(i.0.as_ref(), i.1.as_ref()) {
                    Ok(s) => Some(s),
                    Err(e) => print_err_and_none!(e),
                }
            });

        Ok(Box::new(iter))
    }

    fn since_until(&self, k: &str, k2: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse(&self, k: &str) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse_until(&self, k: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}
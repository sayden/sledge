use anyhow::Error;
use sled::IVec;
use crate::conversions::vector::convert_vec_pairs_u8;
use crate::components::storage::{Storage, SledgeIterator};
use crate::components::kv::KV;
use crate::storage::stats::Stats;

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
    fn get(&self, s: String) -> Result<Option<String>, Error> {
        let db_result = self.db.get(s)?;
        let result = Sled::parse_potential_value(&db_result);
        result
    }

    fn put(&mut self, k: String, v: String) -> Result<(), Error> {
        self.db.insert(k.as_bytes(), v.as_bytes())
            .and_then(|_| Ok(()))
            .or_else(|x| bail!(x))
    }

    fn start(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        let ranged = self.db.scan_prefix("");
        Ok(Box::new(ranged.filter_map(|x| Sled::parse_range(x))))
    }

    fn end(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        unimplemented!()
    }

    fn since(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        let ranged = self.db.range(k..);
        Ok(Box::new(ranged.filter_map(|x| Sled::parse_range(x))))
    }

    fn since_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let result = self.db.range(k1..k2);
        Ok(Box::new(result.filter_map(|x| Sled::parse_range(x))))
    }

    fn reverse(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        let ranged = self.db.range(k..).rev();
        Ok(Box::new(ranged.filter_map(|x| Sled::parse_range(x))))
    }

    fn reverse_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let ranged = self.db.range(k1..k2).rev();
        Ok(Box::new(ranged.filter_map(|x| Sled::parse_range(x))))
    }

    fn stats(&self) -> Stats {
        unimplemented!()
    }
}

impl Sled {
    fn parse_range(item: Result<(IVec, IVec), sled::Error>) -> Option<KV> {
        let i = item.or_else(|e| bail!(e)).unwrap();
        let res: Option<KV> = match convert_vec_pairs_u8(i.0.as_ref(), i.1.as_ref()) {
            Ok(s) => Some(s),
            Err(e) => print_err_and_none!(e),
        };
        res
    }
}
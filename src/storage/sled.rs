use sled::IVec;
use crate::components::{Storage, Bound};
use crate::conversions::vector::convert_vec_pairs_u8;

pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn parse_potential_value(i: &Option<IVec>) -> Result<Option<String>, failure::Error> {
        let value = i.as_ref().ok_or(failure::err_msg("value not found"))?;
        return match std::str::from_utf8(value.as_ref()) {
            Ok(x) => Ok(Some(x.to_string())),
            Err(e) => bail!(e),
        };
    }
}


impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, failure::Error> {
        let db_result = self.db.get(s).or_else(|e| bail!(e)).unwrap();
        let result = Sled::parse_potential_value(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<(), failure::Error> {
        self.db.insert(k, v)
            .and_then(|_| Ok(()))
            .or_else(|x| bail!(x))
    }

    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
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

    fn since(&self, k: &str, bounds: Box<[Bound]>) -> Result<Box<Iterator<Item=(String, String)>>, failure::Error> {
        unimplemented!()
    }

    fn backwards(&self, k: &str, bounds: Box<[Bound]>) -> Result<Box<Iterator<Item=(String, String)>>, failure::Error> {
        unimplemented!()
    }
}
use sled::IVec;

use crate::transformations;
use crate::storage::Storage;
use crate::print_err_and_none;


pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn parse_potential_value(i: &Option<IVec>) -> Result<Option<String>, failure::Error> {
        return match i {
            Some(s) => match String::from_utf8(s.to_vec()) {
                Ok(x) => Ok(Some(x)),
                Err(e) => bail!(e),
            },
            None => Ok(None),
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
                return match item {
                    Ok(i) => {
                        match transformations::convert_vec_pairs(i.0.as_ref().to_vec(), i.0.as_ref().to_vec()) {
                            Ok(s) => Some(s),
                            Err(e) => print_err_and_none!(e),
                        }
                    }
                    Err(e) => print_err_and_none!(e)
                };
            });

        Ok(Box::new(iter))
    }
}
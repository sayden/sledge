use sled::IVec;

use crate::errors::{AppError, ErrorType};
use crate::{errors, transformations};
use crate::storage::Storage;
use crate::{convert_to_ap_error, print_err_and_none};


pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn parse_potential_value(i: &Result<Option<IVec>, sled::Error>) -> Result<Option<String>, AppError> {
        return match i {
            Ok(o) => match o {
                Some(s) => match String::from_utf8(s.to_vec()) {
                    Ok(x) => Ok(Some(x)),
                    Err(e) => convert_to_ap_error!(e, Db),
                },
                None => Ok(None),
            }
            Err(e) => convert_to_ap_error!(e, Db),
        };
    }
}


impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, AppError> {
        let db_result = self.db.get(s);
        let result = Sled::parse_potential_value(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<(), AppError> {
        self.db.insert(k, v)
            .and_then(|_| Ok(()))
            .or_else(|x| Err(errors::new_msg(x.to_string(), ErrorType::Db)))
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
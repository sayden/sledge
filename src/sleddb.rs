use sled::IVec;
use crate::storage::{Storage};
use std::error::Error;
use crate::app_errors::{AppErrorV2, ErrorType};
use crate::app_errors;

pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn asads(i: &Result<Option<IVec>, sled::Error>) -> Result<Option<String>, AppErrorV2> {
        return match i {
            Ok(o) => match o {
                Some(s) => Ok(Some(String::from_utf8(s.to_vec()).unwrap())),
                None => Ok(None),
            }
            Err(e) => Err(app_errors::new_msg(e.description().to_string(), ErrorType::Db)),
        };
    }
}

impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, AppErrorV2> {
        let db_result = self.db.get(s);
        let result = Sled::asads(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, AppErrorV2> {
        Sled::asads(&self.db.insert(k, v))
    }
}
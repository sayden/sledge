use sled::{Db, IVec};
use crate::storage::{Storage, DbError};
use std::error::Error;
use std::borrow::Borrow;
use std::str::Utf8Error;

pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub(crate) fn new(p: String) -> Self {
        let db = sled::open(p).unwrap();
        Sled { db }
    }

    fn asads(i: &Result<Option<IVec>, sled::Error>) -> Result<Option<String>, DbError> {
        return match i {
            Ok(o) => match o {
                Some(s) => Ok(Some(String::from_utf8(s.to_vec()).unwrap())),
                None => Ok(None),
            }
            Err(e) => Err(DbError::new(e.description().to_string())),
        };
    }
}

impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, DbError> {
        let db_result = self.db.get(s);
        let result = Sled::asads(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, DbError> {
        Sled::asads(&self.db.insert(k, v))
    }
}
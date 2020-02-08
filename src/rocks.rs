use crate::storage::{Storage, DbError};
use std::error::Error;
use rocksdb::{DB, Options};

pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub(crate) fn new(path: String) -> Self {
        let db = DB::open_default(path).unwrap();
        Rocks { db }
    }

    fn vec_to_string(ivec: Vec<u8>) -> String {
        String::from_utf8(ivec.to_vec()).unwrap()
    }

    fn asads<E: std::marker::Sized + Error>(i: &Result<Option<Vec<u8>>, E>) -> Result<Option<String>, DbError> {
        return match i {
            Ok(o) => match o {
                Some(v) => {
                    let out = match String::from_utf8(v.to_vec()) {
                        Ok(s) => Ok(Some(s)),
                        Err(e) => Err(DbError::new("vec parsing error".to_string()))
                    };
                    out
                }
                None => Ok(None),
            }
            Err(e) => Err(DbError::new(e.description().to_string())),
        };
    }

    fn resolve_put<E: std::marker::Sized + Error>(r: &Result<(),E>) -> Result<Option<String>, DbError> {
        match r {
            Ok(_) => Ok(Some("ok".to_string())),
            Err(e) => Err(DbError::new(e.description().to_string())),
        }
    }
}

impl Storage for Rocks {
    fn get(&self, s: &str) -> Result<Option<String>, DbError> {
        let result = self.db.get(s);
        Rocks::asads(&result)
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, DbError> {
        let result = &self.db.put(k, v);
        Rocks::resolve_put(result)
    }
}
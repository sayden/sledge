use crate::storage::{Storage};
use std::error::Error;
use rocksdb::{DB};
use crate::app_errors::{AppErrorV2, ErrorType};
use crate::app_errors;

pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub fn new(path: String) -> Box<dyn Storage> {
        let db = DB::open_default(path).unwrap();
        Box::new(Rocks { db })
    }

    fn asads<E: std::marker::Sized + Error>(i: &Result<Option<Vec<u8>>, E>) -> Result<Option<String>, AppErrorV2> {
        return match i {
            Ok(o) => match o {
                Some(v) => {
                    let out = match String::from_utf8(v.to_vec()) {
                        Ok(s) => Ok(Some(s)),
                        Err(e) => Err(app_errors::new_msg(e.to_string(),ErrorType::Db)),
                    };
                    out
                }
                None => Ok(None),
            }
            Err(e) => Err(app_errors::new_msg(e.description().to_string(), ErrorType::Db)),
        };
    }

    fn resolve_put<E: std::marker::Sized + Error>(r: &Result<(),E>) -> Result<Option<String>, AppErrorV2> {
        match r {
            Ok(_) => Ok(Some("ok".to_string())),
            Err(e) => Err(app_errors::new_msg(e.description().to_string(), ErrorType::Db)),
        }
    }
}

impl Storage for Rocks {
    fn get(&self, s: &str) -> Result<Option<String>, AppErrorV2> {
        let result = self.db.get(s);
        Rocks::asads(&result)
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, AppErrorV2> {
        let result = &self.db.put(k, v);
        Rocks::resolve_put(result)
    }
}
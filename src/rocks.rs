use crate::storage::Storage;
use std::error::Error;
use rocksdb::DB;
use crate::errors::{AppError, ErrorType};
use crate::errors;

pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub fn new(path: String) -> Box<dyn Storage> {
        let db = DB::open_default(path).unwrap();
        Box::new(Rocks { db })
    }

    fn asads<E: std::marker::Sized + Error>(i: &Result<Option<Vec<u8>>, E>) -> Result<Option<String>, AppError> {
        return match i {
            Ok(o) => match o {
                Some(v) => return match String::from_utf8(v.to_vec()) {
                    Ok(s) => Ok(Some(s)),
                    Err(e) => Err(errors::new_msg(e.to_string(), ErrorType::Db)),
                },
                None => Ok(None),
            }
            Err(e) => Err(errors::new_msg(e.description().to_string(), ErrorType::Db)),
        };
    }
}

impl Storage for Rocks {
    fn get(&self, s: &str) -> Result<Option<String>, AppError> {
        let result = self.db.get(s);
        Rocks::asads(&result)
    }

    fn put(&self, k: &str, v: &str) -> Result<(), AppError> {
        self.db.put(k, v)
            .or_else(|x| Err(errors::new_msg(x.into_string(), errors::ErrorType::Db)))
    }

    fn range(&self, k: &str) -> Result<Option<Box<dyn Iterator<Item=(String, String)>>>, AppError> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        Ok(Some(Box::new(db_iter.into_iter()
            .map(|(x, y)| {
                (String::from(x.as_bytes()), String::from(x.as_bytes()))
            }))))
    }
}
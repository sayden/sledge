use sled::IVec;
use crate::storage::Storage;
use std::error::Error;
use crate::errors::{AppError, ErrorType};
use crate::errors;

pub struct Sled {
    db: sled::Db,
}

impl Sled {
    pub fn new(p: String) -> Box<dyn Storage> {
        let db = sled::open(p).unwrap();
        Box::new(Sled { db })
    }

    fn asads(i: &Result<Option<IVec>, sled::Error>) -> Result<Option<String>, AppError> {
        return match i {
            Ok(o) => match o {
                Some(s) => Ok(Some(String::from_utf8(s.to_vec()).unwrap())),
                None => Ok(None),
            }
            Err(e) => Err(errors::new_msg(e.description().to_string(), ErrorType::Db)),
        };
    }
}

impl Storage for Sled {
    fn get(&self, s: &str) -> Result<Option<String>, AppError> {
        let db_result = self.db.get(s);
        let result = Sled::asads(&db_result);
        result
    }

    fn put(&self, k: &str, v: &str) -> Result<(), AppError> {
        self.db.insert(k, v)
            .and_then(|_| Ok(()))
            .or_else(|x| Err(errors::new_msg(x.to_string(), ErrorType::Db)))
    }

    fn range(&self, k: &str) -> Result<Option<Box<dyn Iterator<Item=(String, String)>>>, AppError> {
        let res: Result<Option<(IVec, IVec)>, dyn Error> = self.db.get_gt(k);
//        let iter = self.db.get_gt(k);
//        let v = iter.and_then(|o| {
//            o.map(|(x, y)| {
//                let x1 = String::from_utf8(x.to_vec()).unwrap_or_default();
//                let y1 = String::from_utf8(x.to_vec()).unwrap_or_default();
//
//                Box::new((x1,y1))
//            })
//        });
    }
}

struct SledIterator {
    lastKey: &[u8],
}
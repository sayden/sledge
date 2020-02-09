use crate::storage::Storage;
use rocksdb::DB;
use crate::errors::{AppError, ErrorType};
use crate::errors;
use std::string::FromUtf8Error;

pub struct Rocks {
    db: rocksdb::DB,
}

impl Rocks {
    pub fn new(path: String) -> Box<dyn Storage> {
        let db = DB::open_default(path).unwrap();
        Box::new(Rocks { db })
    }
}

impl Storage for Rocks {
    fn get(&self, s: &str) -> Result<Option<String>, AppError> {
        let result = self.db.get(s);
        let a = result
            .or_else(|e| Err(errors::new_msg(e.into_string(), ErrorType::Db)));
        a.and_then(|o| {
            Ok(o.and_then(|r| vec_to_maybe_string(&r)))
        })
    }

    fn put(&self, k: &str, v: &str) -> Result<(), AppError> {
        self.db.put(k, v)
            .or_else(|x| Err(errors::new_msg(x.into_string(), errors::ErrorType::Db)))
    }

    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        let converted_to_string = db_iter
            .map(|(x, y)| {

                let maybe_pair = convert_vec_pairs(x.to_vec(), y.to_vec());
                let pairs = match maybe_pair {
                    Err(e) => {
                        ("".to_string(), "".to_string())
                    }

                    Ok(pair) => pair,
                };

                pairs
            });

        Ok(Box::new(converted_to_string))
    }
}


fn convert_vec_pairs(x: Vec<u8>, y: Vec<u8>) -> Result<(String, String), failure::Error> {
    let x1: Result<String, FromUtf8Error> = String::from_utf8(x.to_vec());
    let y1: Result<String, FromUtf8Error> = String::from_utf8(y.to_vec());

    let (x2, y2) = match (x1, y1) {
        (Ok(x3), Ok(y3)) => (x3, y3),
        (Err(e1), Err(e2)) => bail!(errors::new_msg(format!("{}, {}", e1, e2), ErrorType::Db)),
        (_, Err(e2)) => bail!(e2),
        (Err(e1), _) => bail!(e1),
    };

    Ok((x2, y2))
}


fn vec_to_maybe_string(v: &Vec<u8>) -> Option<String> {
    String::from_utf8(v.to_vec()).err()
        .and_then(|r| None)
}
use crate::storage::Storage;
use rocksdb::DB;
use crate::errors::{AppError, ErrorType};
use crate::{errors, transformations};
use crate::{convert_to_ap_error, print_err_and_none};

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
            Ok(o.and_then(|r| String::from_utf8(r.to_vec()).err()
                .and_then(|r| print_err_and_none!(r))))
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
                let maybe_pair = transformations::convert_vec_pairs(x.to_vec(), y.to_vec());
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
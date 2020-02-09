use sled::IVec;

use crate::errors::{AppError, ErrorType};
use crate::errors;
use crate::storage::Storage;
use std::string::FromUtf8Error;

macro_rules! convert_to_ap_error {
    ($i: expr, $e: ident)=>{
        Err(errors::new_msg(format!("{}", $i), ErrorType::$e))
    }
}

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
            Err(e) => convert_to_ap_error!(e, Db),
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

    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        let ranged_result = self.db.range(k..);

        let mut err: Option<failure::Error> = None;

        let iter = ranged_result
            .map(|item| {
                match item {
                    Err(_) => { ("".to_string(), "".to_string()) }
                    Ok(result) => {
                        let maybe_pair = convert_ivec_pairs(result.0, result.1);
                        let pairs = match maybe_pair {
                            Err(e) => {
                                ("".to_string(), "".to_string())
                            }

                            Ok(pair) => pair,
                        };

                        pairs
                    }
                }
            });

        match err {
            Some(e) => bail!(e),
            None => (),
        }

        Ok(Box::new(iter))
    }
}

fn convert_ivec_pairs(x: IVec, y: IVec) -> Result<(String, String), failure::Error> {
    let x1: Result<String, FromUtf8Error> = String::from_utf8(x.as_ref().to_vec());
    let y1: Result<String, FromUtf8Error> = String::from_utf8(y.as_ref().to_vec());

    let (x2, y2) = match (x1, y1) {
        (Ok(x3), Ok(y3)) => (x3, y3),
        (Err(e1), Err(e2)) => bail!(errors::new_msg(format!("{}, {}", e1, e2), ErrorType::Db)),
        (_, Err(e2)) => bail!(e2),
        (Err(e1), _) => bail!(e1),
    };

    Ok((x2, y2))
}

use crate::storage::Storage;
use rocksdb::DB;
use crate::transformations;
use crate::print_err_and_none;

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
    fn get(&self, s: &str) -> Result<Option<String>, failure::Error> {
        let result = self.db.get(s)?;

        let b = result.and_then(|r| match String::from_utf8(r.to_vec()) {
            Ok(v) => Some(v),
            Err(e) => print_err_and_none!(e)
        });

        Ok(b)
    }

    fn put(&self, k: &str, v: &str) -> Result<(), failure::Error> {
        self.db.put(k, v)
            .or_else(|x| bail!(x))
    }

    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        let db_iter = self.db.iterator(rocksdb::IteratorMode::From(k.as_bytes(),
                                                                   rocksdb::Direction::Forward));
        let converted_to_string = db_iter
            .map(|(x, y)| {
                let maybe_pair = transformations::convert_vec_pairs(x.to_vec(), y.to_vec());
                let pairs = match maybe_pair {
                    Err(e) => print_err_and_none!(e),
                    Ok(pair) => Some(pair),
                };

                pairs
            })
            .filter_map(|x| {
                return if x.is_some() {
                    Some(x.unwrap())
                } else { None };
            });

        Ok(Box::new(converted_to_string))
    }
}
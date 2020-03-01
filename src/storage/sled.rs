use sled::{IVec, Tree};
use crate::components::storage::{Storage, Error, StorageIter};
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::ops::Deref;
use std::env;
use bytes::Bytes;
use crate::server::requests::Query;

pub struct Sled {
    db: sled::Db,
    create_tree_if_missing: bool,
}

impl Sled {
    pub fn new_storage(p: String) -> impl Storage + Send + Sync {
        let db = sled::open(p).unwrap();
        let create_tree = match env::var("FEEDB_CREATE_TREE_IF_MISSING") {
            Ok(res) => res == "true",
            _ => true,
        };
        Sled { db, create_tree_if_missing: create_tree }
    }
}


impl Storage for Sled {
    fn get(&self, maybe_keyspace: Option<String>, s: String) -> Result<String, Error> {
        let tree = self.get_tree(maybe_keyspace)?;

        let db_result = tree.get(s.clone())
            .or_else(|err| Err(Error::Db(err.to_string())))?;

        match db_result {
            Some(v) => std::str::from_utf8(v.as_ref())
                .or_else(|err| Err(Error::Parse(err)))
                .and_then(|res| Ok(res.to_string())),
            None => Err(Error::ValueNotFound(s)),
        }
    }

    fn put(&mut self, maybe_keyspace: Option<String>, k: String, v: Bytes) -> Result<(), Error> {
        let tree = self.get_tree(maybe_keyspace)?;

        tree.insert(k.as_bytes(), v.as_ref())
            .and_then(|_| Ok(()))
            .or_else(|err| Err(Error::Put(err.to_string())))
    }

    fn create_keyspace(&mut self, name: String) -> Result<(), Error> {
        self.db.open_tree(name)
            .or_else(|err| Err(Error::Db(err.to_string())))
            .and(Ok(()))
    }

    fn start(&self, maybe_keyspace: Option<String>) -> Result<StorageIter, Error> {
        unimplemented!()
        // let tree = self.get_tree(maybe_keyspace)?;
        // let ranged = tree.scan_prefix("");
        // Ok(Box::new(ranged.filter_map(Sled::parse_range)))
    }

    fn end(&self, _maybe_keyspace: Option<String>) -> Result<StorageIter, Error> {
        unimplemented!()
    }

    fn range(&self, _keyspace: Option<String>, _query: Query) -> Result<StorageIter, Error> {
        unimplemented!()
    }

    fn since(&self, maybe_keyspace: Option<String>, k: String) -> Result<StorageIter, Error> {
        let tree = self.get_tree(maybe_keyspace)?;
        let ranged = tree.range(k..);
        let iter: Box<dyn Iterator<Item=sled::Result<(IVec, IVec)>> + Send + Sync> = box ranged;
        Ok(Box::new(iter.filter_map(Sled::parse_range)))
    }

    fn since_until(&self, maybe_keyspace: Option<String>, k1: String, k2: String)
                   -> Result<StorageIter, Error> {
        let tree = self.get_tree(maybe_keyspace)?;
        let result = tree.range(k1..k2);
        Ok(Box::new(result.filter_map(Sled::parse_range)))
    }

    fn reverse(&self, maybe_keyspace: Option<String>, k: String) -> Result<StorageIter, Error> {
        let tree = self.get_tree(maybe_keyspace)?;
        let ranged = tree.range(k..).rev();
        Ok(Box::new(ranged.filter_map(Sled::parse_range)))
    }

    fn reverse_until(&self, maybe_keyspace: Option<String>, k1: String, k2: String)
                     -> Result<StorageIter, Error> {
        let tree = self.get_tree(maybe_keyspace)?;
        let ranged = tree.range(k1..k2).rev();
        Ok(Box::new(ranged.filter_map(Sled::parse_range)))
    }

    fn stats(&self) -> Stats {
        unimplemented!()
    }
}

impl Sled {
    fn parse_range(item: Result<(IVec, IVec), sled::Error>) -> Option<KV> {
        let (x, y) = item.ok()?;
        Some(KV { key: x.to_vec(), value: y.to_vec() }) //TODO this seems to copies the entire vector
    }

    fn get_tree(&self, maybe_keyspace: Option<String>) -> Result<Tree, Error> {
        match maybe_keyspace {
            Some(ref ks) => {
                if !self.create_tree_if_missing {
                    let mut found: bool = false;
                    for tree_name in self.db.tree_names() {
                        let res = std::str::from_utf8(tree_name.as_ref())
                            .or_else(|err| Err(Error::Parse(err)))?;
                        if res == ks {
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return Err(Error::NotFound(format!("tree '{}' not found", ks)));
                    }
                }

                self.db.open_tree(ks.as_bytes())
                    .or_else(|err| Err(Error::NotFound(err.to_string())))
            }
            None => Ok(self.db.deref().clone()),
        }
    }
}
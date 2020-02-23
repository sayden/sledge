use crate::components::storage::{Storage, SledgeIterator};
use anyhow::Error;
use crate::components::kv::KV;
use crate::storage::stats::Stats;
use std::collections::HashMap;

pub struct Memory {
    h: HashMap<String, Vec<KV>>,
    total_entries: usize,
}

impl Memory {
    pub fn new<'a>() -> Box<dyn Storage + Send + Sync> {
        let mut hs: HashMap<String, Vec<KV>> = HashMap::new();
        hs.insert("".to_string(), Vec::new());
        Box::new(Memory { h: hs, total_entries: 0 })
    }
}

impl Storage for Memory {
    fn get(&self, maybe_keyspace: Option<String>, k: String) -> Result<Option<String>, Error> {
        let res = maybe_keyspace
            .and_then(|k| self.h.get(&k))
            .ok_or(anyhow!("keyspace error"))?;

        for i in res {
            if i.key == k {
                return Ok(Some(i.value.clone()));
            }
        }

        Ok(None)
    }

    fn put(&mut self, maybe_keyspace: Option<String>, k: String, v: String) -> Result<(), Error> {
        let ks: String = match maybe_keyspace {
            Some(k) => k,
            None => "".to_string(),
        };

        let vc = self.h.get(&ks);
        let kv = KV { key: k, value: v };

        let mut inner_vec = match vc {
            None => Vec::new(),
            Some(vc) => vc.clone()
        };

        inner_vec.push(kv);
        inner_vec.sort();
        self.h.insert(ks, inner_vec);
        self.total_entries += 1;

        Ok(())
    }

    fn start(&self, maybe_keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV>>, anyhow::Error> {
        let res = maybe_keyspace.and_then(|k| self.h.get(&k))
            .ok_or(anyhow!("keyspace error"))
            .and_then(|x| Ok(x.clone().into_iter()))?;

        Ok(Box::new(res))
    }

    fn end(&self, _keyspace: Option<String>) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        unimplemented!()
    }

    fn since(&self, maybe_keyspace: Option<String>, k: String) -> Result<Box<SledgeIterator>, Error> {
        let res = maybe_keyspace
            .and_then(|k| self.h.get(&k))
            .ok_or(anyhow!("keyspace error"))
            .and_then(|vc| Ok(vc.clone()
                .into_iter()
                .skip_while(move |x| (*x) != k)))?;

        Ok(Box::new(res))
    }

    fn since_until(&self, _keyspace: Option<String>, _since_key: String, _to_key: String) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse(&self, maybe_keyspace: Option<String>, k1: String) -> Result<Box<SledgeIterator>, Error> {
        let res = maybe_keyspace.and_then(|k| self.h.get(&k))
            .ok_or(anyhow!("keyspace error"))
            .and_then(|x|
                Ok(x.clone()
                    .into_iter()
                    .rev()
                    .skip_while(|x| (*x) != k1)))?;

        Ok(Box::new(res.collect::<Vec<KV>>().into_iter()))
    }

    fn reverse_until(&self, maybe_keyspace: Option<String>, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let res = maybe_keyspace.and_then(|k| self.h.get(&k))
            .ok_or(anyhow!("keyspace error"))
            .and_then(|x|
                Ok(x.clone()
                    .into_iter()
                    .rev()
                    .skip_while(move |x| (*x) != k1)
                    .take_while(move |x| (*x) != k2)))?;

        Ok(Box::new(res.collect::<Vec<KV>>().into_iter()))
    }

    fn stats(&self) -> Stats {
        Stats {
            total_entries: self.total_entries as u128,
            error: "".to_string(),
        }
    }
}

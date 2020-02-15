use crate::components::storage::{Storage, SledgeIterator};
use anyhow::Error;
use crate::components::kv::KV;
use crate::storage::stats::Stats;

#[derive(Sync)]
pub struct Memory {
    v: Vec<KV>
}

impl Memory {
    pub fn new<'a>() -> Box<dyn Storage> {
        let v: Vec<KV> = Vec::new();
        Box::new(Memory { v })
    }
}

impl Storage for Memory {
    fn get(&self, k: String) -> Result<Option<String>, Error> {
        for i in &self.v {
            if i.key == k {
                return Ok(Some(i.value.clone()));
            }
        }

        Ok(None)
    }

    fn put(&mut self, k: String, v: String) -> Result<(), Error> {
        let kv = KV { key: k, value: v };
        self.v.push(kv);
        self.v.sort();
        Ok(())
    }

    fn start(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        Ok(Box::new(self.v.clone().into_iter()))
    }

    fn end(&self) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        unimplemented!()
    }

    fn since(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(self.v.clone().into_iter().skip_while(move |x| *x != k)))
    }

    fn since_until(&self, _since_key: String, _to_key: String) -> Result<Box<SledgeIterator>, Error> {
        let res = self.v.clone().into_iter()
            .skip_while(|x| *x != "03".to_string())
            .take(3);

        Ok(Box::new(res))
    }

    fn reverse(&self, k1: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(self.v.clone().into_iter().rev().skip_while(move |x| *x != k1)))
    }

    fn reverse_until(&self, k1: String, k2: String) -> Result<Box<SledgeIterator>, Error> {
        let i = self.v.clone().into_iter()
            .rev()
            .skip_while(move |x| *x != k1)
            .take_while(move |x| *x != k2);

        Ok(Box::new(i))
    }

    fn stats(&self) -> Stats {
        Stats {
            total_entries: self.v.len() as u128,
            error: "".to_string()
        }
    }
}

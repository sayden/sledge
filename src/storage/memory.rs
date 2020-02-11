use crate::components::storage::{Storage, Options, KV, SledgeIterator};
use anyhow::Error;
use crate::storage::options::UntilExt;


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
        Ok(())
    }

    fn since(&self, _: String) -> Result<Box<SledgeIterator>, Error> {
        let v = vec![1, 2, 3, 4, 5].into_iter();
        Ok(Box::new(v.map(|x| (KV { key: format!("{}", x), value: format!("{}", x) }))))
    }

    fn since_until(&self, _since_key: String, to_key: String, _opt: Option<Vec<Options>>) -> Result<Box<SledgeIterator>, Error> {
        let found = KV { key: to_key.to_string(), value: "".to_string() };
        let res = self.v.clone().into_iter().until(found);

        Ok(Box::new(res))
    }

    fn reverse(&self, _k: String) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse_until(&self, _k: String, _opt: Option<Vec<Options>>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}

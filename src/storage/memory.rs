use crate::components::storage::{Storage, SledgeIterator};
use anyhow::Error;
use crate::components::kv::KV;


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

    fn since(&self, _: String) -> Result<Box<SledgeIterator>, Error> {
        let v = vec![1, 2, 3, 4, 5].into_iter();
        Ok(Box::new(v.map(|x| (KV { key: format!("{}", x), value: format!("{}", x) }))))
    }

    fn since_until(&self, _since_key: String, _to_key: String) -> Result<Box<SledgeIterator>, Error> {
        let res = self.v.clone().into_iter()
            .skip_while(|x| *x != "03".to_string())
            .take(3)
//            .until(Box::new(Until::new("04".to_string())))
            ;

        Ok(Box::new(res))
    }

    fn reverse(&self, _k: String) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse_until(&self, _k: String) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}

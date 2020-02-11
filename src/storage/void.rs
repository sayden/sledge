use crate::components::storage::{Storage, Options, KV, SledgeIterator};
use anyhow::Error;

pub struct Void {}

impl Void {
    pub fn new() -> Box<dyn Storage> {
        Box::new(Void {})
    }
}

impl Storage for Void {
    fn get(&self, _: &str) -> Result<Option<String>, Error> {
        Ok(Some("void get".to_string()))
    }

    fn put(&self, _: &str, _: &str) -> Result<(), Error> {
        Ok(())
    }

    fn since(&self, _: &str) -> Result<Box<SledgeIterator>, Error> {
        let v = vec![1, 2, 3, 4, 5].into_iter();
        Ok(Box::new(v.map(|x| (KV { key: format!("{}", x), value: format!("{}", x) }))))
    }

    fn since_until(&self, k: &str, k2: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse(&self, k: &str) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse_until(&self, k: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}
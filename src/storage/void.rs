use crate::components::storage::{Storage, SledgeIterator};
use anyhow::Error;
use crate::components::kv::KV;
use crate::storage::stats::Stats;

pub struct Void {}

impl Void {
    pub fn new() -> Box<dyn Storage + Send + Sync> {
        Box::new(Void {})
    }
}

impl Storage for Void {
    fn get(&self, _: Option<String>, _: String) -> Result<Option<String>, Error> {
        Ok(Some("void get".to_string()))
    }

    fn put(&mut self, _: Option<String>, _: String, _: String) -> Result<(), Error> {
        Ok(())
    }

    fn start(&self, _: Option<String>) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn end(&self, _: Option<String>) -> Result<Box<dyn Iterator<Item=KV>>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn since(&self, _: Option<String>, _: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn since_until(&self, _: Option<String>, _: String, _: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn reverse(&self, _: Option<String>, _: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn reverse_until(&self, _: Option<String>, _: String, _: String) -> Result<Box<SledgeIterator>, Error> {
        Ok(Box::new(vec![].into_iter()))
    }

    fn stats(&self) -> Stats {
        unimplemented!()
    }
}

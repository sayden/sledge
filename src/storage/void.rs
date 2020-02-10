use crate::components::{Storage, Bound};
use failure::Error;

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

    fn range(&self, _: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, Error> {
        let v = vec![1, 2, 3, 4, 5].into_iter();
        Ok(Box::new(v.map(|x| (format!("{}", x), format!("{}", x)))))
    }

    fn since(&self, k: &str, bounds: Box<[Bound]>) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        unimplemented!()
    }

    fn backwards(&self, k: &str, bounds: Box<[Bound]>) -> Result<Box<dyn Iterator<Item=(String, String)>>, failure::Error> {
        unimplemented!()
    }
}
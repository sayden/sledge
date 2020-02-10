pub mod framework;
pub mod api;

use failure::{Error};

pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, Error>;
    fn put(&self, k: &str, v: &str) -> Result<(), Error>;
    fn range(&self, k: &str) -> Result<Box<dyn Iterator<Item=(String, String)>>, Error>;
}
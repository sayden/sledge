use core::fmt;
use std::error;
//use crate::sleddb::Sled;

pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, DbError>;
    fn put(&self, k: &str, v: &str) -> Result<Option<String>, DbError>;
}

#[derive(Debug)]
pub struct DbError { msg: String }

impl DbError {
    pub fn new(s: String) -> DbError {
        DbError { msg: s }
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for DbError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
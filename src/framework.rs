use std::error;
use core::fmt;
use crate::storage::Storage;
use std::error::Error;

pub trait Framework {
    fn get(&self, k: &str) -> Result<Option<String>, FrameworkError>;
    fn put(&self, k: &str, v: &str) -> Result<Option<String>, FrameworkError>;
}

pub struct FrameworkV1 {
    pub storage: Box<dyn Storage>
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<Option<String>, FrameworkError> {
        match self.storage.get(s) {
            Ok(o) => match o {
                Some(r) => Ok(Some(r)),
                None => Ok(None),
            },
            Err(e) => Err(FrameworkError::new(String::from(e.description()))),
        }
    }

    fn put(&self, k: &str, v: &str) -> Result<Option<String>, FrameworkError> {
        match self.storage.put(k,v) {
            Ok(o) => match o {
                Some(r) => Ok(Some(r)),
                None => Ok(None),
            },
            Err(e) => Err(FrameworkError::new(String::from(e.description()))),
        }
    }
}

pub fn new(s: Box<dyn Storage>) -> Box<dyn Framework> {
    Box::new(FrameworkV1 { storage: s })
}

#[derive(Debug)]
pub struct FrameworkError { msg: String }

impl FrameworkError {
    pub fn new(s: String) -> FrameworkError {
        FrameworkError { msg: s }
    }
}

impl fmt::Display for FrameworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for FrameworkError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
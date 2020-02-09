use crate::framework::Framework;
use core::fmt;
use std::error;
use std::error::Error;

pub trait App {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, AppError>;
}

struct V1 {
    framework: Box<dyn Framework>
}

pub fn new(f: Box<dyn Framework>) -> Box<dyn App> {
    return Box::new(V1 { framework: f });
}

impl App for V1 {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, AppError> {
        return match self.framework.get(k) {
            Ok(o) => Ok(o),
            Err(e) => Err(AppError::new(e.description().to_string())),
        };
    }
}

#[derive(Debug)]
pub struct AppError { msg: String }

impl AppError {
    pub fn new(s: String) -> AppError {
        AppError { msg: s }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for AppError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
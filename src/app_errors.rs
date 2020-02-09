use core::fmt;
use std::fmt::{Display, Formatter, Error};
use std::error;

#[derive(Debug)]
pub enum ErrorType {
    App,
    Framework,
    Db,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

trait NewError<T> {
    fn new(e: T, t: ErrorType) -> AppErrorV2;
}


pub fn new_msg(s: String, t: ErrorType) -> AppErrorV2 {
    AppErrorV2 { msg: s, error_type: t }
}

pub fn new_error(e: Error, t: ErrorType) -> AppErrorV2 {
    AppErrorV2 { msg: e.to_string(), error_type: t }
}

#[derive(Debug)]
pub struct AppErrorV2 { msg: String, error_type: ErrorType }


impl fmt::Display for AppErrorV2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} error: {}", self.error_type, self.msg)
    }
}

impl error::Error for AppErrorV2 {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

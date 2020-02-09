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
    fn new(e: T, t: ErrorType) -> AppError;
}


pub fn new_msg(s: String, t: ErrorType) -> AppError {
    AppError { msg: s, error_type: t }
}

#[derive(Debug)]
pub struct AppError { msg: String, error_type: ErrorType }


impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} error: {}", self.error_type, self.msg)
    }
}

impl error::Error for AppError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

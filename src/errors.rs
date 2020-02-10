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

#[macro_export]
macro_rules! print_err_and_none {
    ($e:expr) => {{
        {
            println!("{}", $e);
            None
        }
    }}
}

#[macro_export]
macro_rules! convert_to_ap_error {
    ($i: expr, $e: ident)=>{
        Err(errors::new_msg(format!("{}", $i), ErrorType::$e))
    }
}
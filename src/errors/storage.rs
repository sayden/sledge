use std::{fmt, error};
use std::fmt::{Formatter};

#[derive(Debug)]
enum StorageError{
    SledError(sled::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            StorageError::SledError(ref err) => write!(f, "sled error '{}'", err),
        }
    }
}

impl error::Error for StorageError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            StorageError::SledError( ref err) => Some(err),
        }
    }
}

impl From<sled::Error> for StorageError {
    fn from(err: sled::Error) -> StorageError {
        StorageError::SledError(err)
    }
}
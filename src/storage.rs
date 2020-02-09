use crate::errors::AppError;

pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, AppError>;
    fn put(&self, k: &str, v: &str) -> Result<(), AppError>;
    fn range(&self, k: &str) -> Result<Option<Box<dyn Iterator<Item=(String, String)>>>, AppError>;
}
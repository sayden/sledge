use crate::app_errors::AppErrorV2;

pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, AppErrorV2>;
    fn put(&self, k: &str, v: &str) -> Result<Option<String>, AppErrorV2>;
}
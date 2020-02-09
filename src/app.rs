use crate::framework::Framework;
use crate::app_errors::{AppErrorV2};


pub trait App {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, AppErrorV2>;
}

struct V1 {
    framework: Box<dyn Framework>
}

pub fn new(f: Box<dyn Framework>) -> Box<dyn App> {
    return Box::new(V1 { framework: f });
}

impl App for V1 {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, AppErrorV2> {
        self.framework.get(k)
    }
}

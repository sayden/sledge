use crate::components::Storage;


pub trait App {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, failure::Error>;
//    fn get_since(&self, k: &str, limit: u32) -> Result<Option<dyn Iterator>, failure::Error>;
}

struct V1 {
    storage: Box<dyn Storage>
}

pub fn new(f: Box<dyn Storage>) -> Box<dyn App> {
    return Box::new(V1 { storage: f });
}

impl App for V1 {
    fn get_by_id(&self, k: &str) -> Result<Option<String>, failure::Error> {
        self.storage.get(k)
    }

//    fn get_since(&self, k: &str, limit: u32) -> Result<Option<dyn Iterator<Item=_>>, AppError> {
//        unimplemented!()
//    }
}

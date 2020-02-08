pub trait Framework {
    fn get(&self, s: &str) -> Result<String, String>;
}

struct FrameworkV1 {
    storage: dyn StorageV1
}

impl Framework for FrameworkV1 {
    fn get(&self, s: &str) -> Result<String, String> {
        unimplemented!()
    }
}
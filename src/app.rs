
pub trait App {
    fn get_by_id(&self, s: &str) -> Result<String, String>;
}

struct V1 {
    framework: FrameworkV1
}

impl App for V1 {
    fn get_by_id(&self, s: &str) -> Result<String, String> {
        unimplemented!()
    }
}
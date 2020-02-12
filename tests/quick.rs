#[cfg(test)]
mod quick {
    use sledge::components::kv::KV;
    use sledge::storage::options::{After, ProcessOrStop};
    use std::fmt::Debug;

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct A {
        key: String
    }

    impl PartialEq<String> for A {
        fn eq(&self, other: &String) -> bool {
            self.key == *other
        }
    }

    #[test]
    fn test_after_key() {
        let v = vec![1,2,3,4,5];

//        v.into_iter().
    }

}
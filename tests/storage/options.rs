#[cfg(test)]
mod process_or_stop {
    use sledge::components::kv::KV;
    use sledge::storage::options::{After, ProcessOrStop};

    struct B {
        key: String
    }

    impl PartialEq<String> for B {
        fn eq(&self, other: &String) -> bool {
            self.key == *other
        }
    }

    #[test]
    fn test_after() {
        let mut p = After::new("hello".to_string());
        let res = p.process_or_stop(KV { key: "hello".to_string(), value: "world".to_string() });
        assert_eq!(res, Some(KV { key: "hello".to_string(), value: "world".to_string() }));

        let mut p = After::new("".to_string());
        let res = p.process_or_stop(KV { key: "hello".to_string(), value: "world".to_string() });
        assert_eq!(res, None);

        let mut p = After::new("world".to_string());
        let res = p.process_or_stop(KV { key: "hello".to_string(), value: "".to_string() });
        assert_eq!(res, None);

        let mut p = After::new(KV { key: "hello".to_string(), value: "".to_string() });
        let res = p.process_or_stop(KV { key: "hello".to_string(), value: "".to_string() });
        assert_eq!(res, Some(KV { key: "hello".to_string(), value: "".to_string() }));

        let mut p = After::new(KV { key: "hello".to_string(), value: "".to_string() });
        let res = p.process_or_stop(KV { key: "hello".to_string(), value: "world".to_string() });
        assert_eq!(res, None)
    }
}
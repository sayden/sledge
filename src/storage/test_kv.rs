#[cfg(test)]
use crate::components::kv::KV;

#[test]
fn test_string_comparison() {
    let x = KV { key: "Hello".to_string(), value: "World".to_string() };
    assert_eq!(x, "Hello".to_string());
    assert_eq!(x, KV { key: "Hello".to_string(), value: "World".to_string() });
    assert_ne!(x, KV { key: "".to_string(), value: "World".to_string() });
    assert_ne!(x, KV { key: "Hello".to_string(), value: "".to_string() });
}

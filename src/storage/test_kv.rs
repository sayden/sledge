#[cfg(test)]
use crate::components::kv::KV;

#[test]
fn test_string_comparison() {
    let x = KV { key: "Hello".as_bytes().to_vec(), value: "World".as_bytes().to_vec() };
    assert_eq!(x, "Hello".to_string());
    assert_eq!(x, KV { key: "Hello".as_bytes().to_vec(), value: "World".as_bytes().to_vec() });
    assert_ne!(x, KV { key: "".as_bytes().to_vec(), value: "World".as_bytes().to_vec() });
    assert_ne!(x, KV { key: "Hello".as_bytes().to_vec(), value: "".as_bytes().to_vec() });
}

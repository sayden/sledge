#[cfg(test)]
use crate::components::kv::KV;
#[cfg(test)]
use crate::conversions::vector::convert_vec_pairs_u8;

#[test]
fn test_convert_vec_pairs_u8() {
    let s1 = "hello";
    let s2 = "world";
    let res = convert_vec_pairs_u8(s1.as_bytes(), s2.as_bytes());
    assert_eq!(res.unwrap(), KV { key: String::from(s1), value: String::from(s2) })
}
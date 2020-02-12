use anyhow::Error;
use crate::components::kv::KV;

pub fn convert_vec_pairs(x: Vec<u8>, y: Vec<u8>) -> Result<KV, Error> {
    let x1 = String::from_utf8(x)?;
    let y1 = String::from_utf8(y)?;

    Ok(KV { key: x1, value: y1 })
}

pub fn convert_vec_pairs_u8(x: &[u8], y: &[u8]) -> Result<KV, Error> {
    let x1 = std::str::from_utf8(x)?;
    let y1 = std::str::from_utf8(y)?;

    Ok(KV { key: x1.to_string(), value: y1.to_string() })
}

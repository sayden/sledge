use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub struct SimplePair {
    pub id: Vec<u8>,
    pub value: Vec<u8>,
}

impl SimplePair {
    pub fn new_u8(k: &[u8], v: &[u8]) -> Self {
        SimplePair {
            id: k.to_vec(),
            value: v.to_vec(),
        }
    }

    pub fn new_boxed(b: (Box<[u8]>, Box<[u8]>)) -> Self {
        SimplePair {
            id: b.0.to_vec(),
            value: b.1.to_vec(),
        }
    }

    pub fn new_str_vec(k: &str, v: Vec<u8>) -> Self {
        SimplePair {
            id: Vec::from(k),
            value: v,
        }
    }

    pub fn new_vec(k: Vec<u8>, v: Vec<u8>) -> Self {
        SimplePair { id: k, value: v }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimplePairJSON {
    id: String,
    val: Box<Value>,
}

pub fn simple_pair_to_json(sp: SimplePair, include_id: bool) -> Option<Value> {
    let v: Value = serde_json::from_slice(sp.value.as_slice())
        .map_err(|err| {
            log::warn!(
                "error trying to convert 'value' to string: {}",
                err.to_string()
            )
        })
        .ok()?;

    if include_id {
        let k = String::from_utf8(sp.id)
            .map_err(|err| log::warn!("error trying to get json from 'key': {}", err.to_string()))
            .ok()?;

        let res: Value = serde_json::to_value(&SimplePairJSON { id: k, val: box v })
            .map_err(|err| {
                log::warn!(
                    "error trying to get json from simpleJSON: {}",
                    err.to_string()
                )
            })
            .ok()?;

        return Some(res);
    }

    Some(v)
}

pub struct KvUTF8 {
    pub id: String,
    pub value: String,
}

impl KvUTF8 {
    pub fn default() -> Self {
        KvUTF8 {
            id: "".to_string(),
            value: "".to_string(),
        }
    }
}

impl From<SimplePair> for Option<KvUTF8> {
    fn from(s: SimplePair) -> Self {
        let id = String::from_utf8(s.id)
            .map_err(|err| {
                log::error!(
                    "error trying to convert 'id' to string, an empty value will be used: {}",
                    err
                )
            })
            .ok()?;
        let value = String::from_utf8(s.value)
            .map_err(|err| {
                log::error!(
                    "error trying to convert 'value' to string, an empty value will be used: {}",
                    err
                )
            })
            .ok()?;

        Some(KvUTF8 { id, value })
    }
}

impl From<SimplePair> for KvUTF8 {
    fn from(s: SimplePair) -> Self {
        let maybe_kv: Option<KvUTF8> = From::from(s);
        match maybe_kv {
            Some(v) => v,
            None => KvUTF8::default(),
        }
    }
}

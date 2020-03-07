use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use uuid::adapter::Simple;

use crate::components::errors::Error;
use crate::server::responses::ToMaybeString;

pub struct SimplePair {
    pub k: Vec<u8>,
    pub v: Vec<u8>,
}

impl SimplePair {
    pub fn new_u8(k: &[u8], v: &[u8]) -> Self {
        SimplePair {
            k: k.to_vec(),
            v: v.to_vec(),
        }
    }

    pub fn new_boxed(b: (Box<[u8]>, Box<[u8]>)) -> Self {
        SimplePair {
            k: b.0.to_vec(),
            v: b.1.to_vec(),
        }
    }

    pub fn new_str_vec(k: &str, v: Vec<u8>) -> Self {
        SimplePair { k: Vec::from(k), v }
    }

    pub fn new_vec(k: Vec<u8>, v: Vec<u8>) -> Self {
        SimplePair { k, v }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimplePairJSON {
    k: String,
    v: Box<Value>,
}

pub fn simple_pair_to_json(sp: SimplePair, include_key: bool) -> Option<Value> {
    let v: Value = serde_json::from_slice(sp.v.as_slice())
        .map_err(|err| log::warn!("error trying to convert 'value' to string: {}", err.to_string()))
        .ok()?;

    if include_key {
        let k = String::from_utf8(sp.k)
            .map_err(|err| log::warn!("error trying to get json from 'key': {}", err.to_string()))
            .ok()?;

        let res: Value = serde_json::to_value(&SimplePairJSON { k, v: box v })
            .map_err(|err| log::warn!("error trying to get json from simpleJSON: {}", err.to_string()))
            .ok()?;

        return Some(res)
    }

    Some(v)
}
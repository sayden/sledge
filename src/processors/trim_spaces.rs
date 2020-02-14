use crate::processors::core::Modifier;
use crate::processors::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct TrimSpaces {
    pub modifier: Processor,
}

impl Modifier for TrimSpaces {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);

        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let s: &String = match value {
            Value::String(x) => x,
            _ => return Some(anyhow!("value '{}' is not an string", self.modifier.field))
        };

        v[self.modifier.field.as_str()] = Value::from(s.trim());

        None
    }
}

impl fmt::Display for TrimSpaces {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "TrimSpaces to field '{}'", self.modifier.field)
    }
}
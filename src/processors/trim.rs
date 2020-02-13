use crate::processors::chain::ModifierTrait;
use crate::processors::chain::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Trim {
    pub modifier: Modifier,
    pub from: String,
    pub total: usize,
}

impl ModifierTrait for Trim {
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

        match self.from.as_str() {
            "right" => v[self.modifier.field.as_str()] = Value::from(s.split_at(self.total).1),
            _ => v[self.modifier.field.as_str()] = Value::from(s.split_at(self.total).0),
        }

        None
    }
}

impl fmt::Display for Trim {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Trim string '{}' {} chars from {}", self.modifier.field, self.total, self.from)
    }
}
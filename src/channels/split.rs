use crate::channels::core::Mutator;
use crate::channels::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Split {
    pub modifier: Mutation,
    pub separator: String,
}

impl Mutator for Split {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);

        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let s = match value {
            Value::String(ar) => ar,
            _ => return Some(anyhow!("value '{}' is not an string", self.modifier.field))
        };

        let separator = self.separator.clone();
        if separator.is_empty() {
            return Some(anyhow!("separator cannot be empty"))
        }

        let new_value: Vec<&str> = s.split(&separator).collect();

        v[self.modifier.field.as_str()] = Value::from(new_value);

        None
    }
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Split field: '{}' by '{}'", self.modifier.field, self.separator)
    }
}
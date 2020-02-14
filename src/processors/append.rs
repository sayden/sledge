use crate::processors::core::Modifier;
use crate::processors::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Append {
    pub modifier: Processor,
    pub append: String,
}

impl Modifier for Append {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_field = v.get(&self.modifier.field);

        let value = match maybe_field {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let new_value = format!("{}{}", value.as_str()?, self.append);
        v[&self.modifier.field] = Value::from(new_value);
        None
    }
}

impl fmt::Display for Append {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Append '{}' to field: '{}'", self.append, self.modifier.field)
    }
}
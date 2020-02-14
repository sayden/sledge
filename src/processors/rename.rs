use crate::processors::core::Modifier;
use crate::processors::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Rename {
    pub modifier: Processor,
    pub rename: String,
}

impl Modifier for Rename {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);
        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let new_value = value.clone();
        v.remove(self.modifier.field.as_str())?;
        v.insert(self.rename.clone(), new_value);

        None
    }
}

impl fmt::Display for Rename {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Rename '{}' to field: '{}'", self.rename, self.modifier.field)
    }
}
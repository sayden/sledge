use crate::processors::chain::ModifierTrait;
use crate::processors::chain::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Rename {
    pub modifier: Modifier,
    pub rename: String,
}

impl ModifierTrait for Rename {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);

        match maybe_value {
            Some(value) => {
                let new_value = value.clone();
                v.remove(self.modifier.field.as_str())?;
                v.insert(self.rename.clone(), new_value);
                None
            }
            None => Some(anyhow!("value '{}' not found", self.modifier.field)),
        }
    }
}

impl fmt::Display for Rename {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Rename '{}' to field: '{}'", self.rename, self.modifier.field)
    }
}
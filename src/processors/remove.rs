use crate::processors::core::{ModifierTrait, Modifier};
use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Remove {
    pub(crate) modifier: Modifier
}

impl ModifierTrait for Remove {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        match v.remove(self.modifier.field.as_str()) {
            None => Some(anyhow!("value {} not found", self.modifier.field)),
            Some(_) => None
        }
    }
}

impl fmt::Display for Remove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Remove field: '{}'", self.modifier.field)
    }
}
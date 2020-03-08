use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Rename {
    pub modifier: Mutation,
    pub rename: String,
}

impl Mutator for Rename {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
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

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Rename
    }
}

impl fmt::Display for Rename {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Rename '{}' to field: '{}'", self.rename, self.modifier.field)
    }
}
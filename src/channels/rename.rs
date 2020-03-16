use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

#[derive(Debug)]
pub struct Rename {
    pub modifier: Mutation,
    pub rename: String,
}

impl Mutator for Rename {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v
            .get(&self.modifier.field)
            .ok_or_else(|| Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let new_value = value.clone();
        let _ = v
            .remove(self.modifier.field.as_str())
            .ok_or_else(|| Error::CannotRemoveField(self.modifier.field.to_string()))?;
        v.insert(self.rename.clone(), new_value);

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType { MutatorType::Rename }
}

impl fmt::Display for Rename {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Rename '{}' to field: '{}'",
            self.rename, self.modifier.field
        )
    }
}

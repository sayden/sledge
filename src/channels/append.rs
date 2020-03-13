use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use crate::channels::error::Error;

#[derive(Debug)]
pub struct Append {
    pub modifier: Mutation,
    pub append: String,
}

impl Mutator for Append {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v.get(&self.modifier.field)
            .ok_or(Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?
            .as_str()
            .ok_or(Error::NotString(self.modifier.field.to_string()))?;

        let result = format!("{}{}", value, self.append);
        v[&self.modifier.field] = Value::from(result);
        Ok(())
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Append
    }
}

impl fmt::Display for Append {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Append '{}' to field: '{}'", self.append, self.modifier.field)
    }
}
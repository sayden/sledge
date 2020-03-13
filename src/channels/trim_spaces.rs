use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use crate::channels::error::Error;

#[derive(Debug)]
pub struct TrimSpaces {
    pub modifier: Mutation,
}

impl Mutator for TrimSpaces {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v.get(&self.modifier.field)
            .ok_or(Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let s: &String = match value {
            Value::String(x) => x,
            _ => return Error::NotString(self.modifier.field.to_string()).into()
        };

        v[self.modifier.field.as_str()] = Value::from(s.trim());

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::TrimSpace
    }
}

impl fmt::Display for TrimSpaces {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "TrimSpaces to field '{}'", self.modifier.field)
    }
}
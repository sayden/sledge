use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

#[derive(Debug)]
pub struct Trim {
    pub modifier: Mutation,
    pub from: String,
    pub total: usize,
}

impl Mutator for Trim {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v
            .get(&self.modifier.field)
            .ok_or_else(|| Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let s = match value {
            Value::String(x) => x,
            _ => return Error::NotString(self.modifier.field.to_string()).into(),
        };

        match self.from.as_str() {
            "right" => v[self.modifier.field.as_str()] = Value::from(s.split_at(self.total).1),
            _ => v[self.modifier.field.as_str()] = Value::from(s.split_at(self.total).0),
        }

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType { MutatorType::Trim }
}

impl fmt::Display for Trim {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Trim string '{}' {} chars from {}",
            self.modifier.field, self.total, self.from
        )
    }
}

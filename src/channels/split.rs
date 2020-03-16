use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

#[derive(Debug)]
pub struct Split {
    pub modifier: Mutation,
    pub separator: String,
}

impl Mutator for Split {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v
            .get(&self.modifier.field)
            .ok_or_else(|| Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let s = match value {
            Value::String(ar) => Ok(ar),
            _ => Err(Error::NotString(self.modifier.field.to_string())),
        }?;

        let separator = self.separator.clone();
        if separator.is_empty() {
            return Error::SplitEmptySeparator.into();
        }

        let new_value: Vec<&str> = s.split(&separator).collect();

        v[self.modifier.field.as_str()] = Value::from(new_value);

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType { MutatorType::Split }
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Split field: '{}' by '{}'",
            self.modifier.field, self.separator
        )
    }
}

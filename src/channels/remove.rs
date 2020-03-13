use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::{Mutation, Mutator, MutatorType};

#[derive(Debug)]
pub struct Remove {
    pub(crate) modifier: Mutation
}

impl Mutator for Remove {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        match v.remove(self.modifier.field.as_str()) {
            None => Error::FieldNotFoundInJSON(self.modifier.field.to_string()).into(),
            Some(_) => Ok(())
        }
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Remove
    }
}

impl fmt::Display for Remove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Remove field: '{}'", self.modifier.field)
    }
}
use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::*;
use crate::channels::mutators::Mutator;

#[derive(Debug)]
pub struct Set {
    pub modifier: Mutation,
    pub value: Value,
}

impl Mutator for Set {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        v.insert(self.modifier.field.clone(), self.value.clone());

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Set
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Set '{}' to value: '{}'", self.modifier.field, self.value)
    }
}
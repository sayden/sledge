use crate::channels::core::Mutator;
use crate::channels::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Set {
    pub modifier: Mutation,
    pub value: Value,
}

impl Mutator for Set {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        v.insert(self.modifier.field.clone(), self.value.clone());

        None
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Set
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Set '{}' to value: '{}'", self.modifier.field, self.value)
    }
}
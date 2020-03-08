use crate::channels::mutators::{Mutator, Mutation, MutatorType};
use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Remove {
    pub(crate) modifier: Mutation
}

impl Mutator for Remove {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        match v.remove(self.modifier.field.as_str()) {
            None => Some(anyhow!("value {} not found", self.modifier.field)),
            Some(_) => None
        }
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Remove
    }
}

impl fmt::Display for Remove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Remove field: '{}'", self.modifier.field)
    }
}
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Append {
    pub modifier: Mutation,
    pub append: String,
}

impl Mutator for Append {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_field = v.get(&self.modifier.field);

        let value = match maybe_field {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let new_value = format!("{}{}", value.as_str()?, self.append);
        v[&self.modifier.field] = Value::from(new_value);
        None
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Append
    }
}

impl fmt::Display for Append {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Append '{}' to field: '{}'", self.append, self.modifier.field)
    }
}
use crate::channels::core::Mutator;
use crate::channels::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

pub struct UpperLowercase {
    pub modifier: Mutation,
    pub f: fn(&str) -> String,
}

impl Mutator for UpperLowercase {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);

        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let result = match value {
            Value::Array(ar) => {
                let new_value = ar.into_iter()
                    .filter_map(|x| {
                        match x {
                            Value::String(s) => Some(s.to_lowercase()),
                            _ => None,
                        }
                    })
                    .collect::<Vec<String>>();

                Value::from(new_value)
            }
            Value::String(s) => Value::from((self.f)(s)),
            _ => return Some(anyhow!("value '{}' not found", self.modifier.field)),
        };

        v[&self.modifier.field] = result;

        None
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Uppercase
    }
}

impl fmt::Display for UpperLowercase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "UpperLowercase field: '{}'", self.modifier.field)
    }
}
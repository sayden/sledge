use std::fmt;

use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::*;
use crate::channels::mutators::Mutator;

pub struct UpperLowercase {
    pub modifier: Mutation,
    pub f: fn(&str) -> String,
}

impl Mutator for UpperLowercase {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v.get(&self.modifier.field)
            .ok_or(Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let result = match value {
            Value::Array(ar) => {
                let new_value = ar.iter()
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
            _ => return Error::UpperLowerCaseErrorTypeNotRecognized.into(),
        };

        v[&self.modifier.field] = result;

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Uppercase
    }
}

impl fmt::Display for UpperLowercase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "UpperLowercase field: '{}'", self.modifier.field)
    }
}
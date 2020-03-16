use std::fmt;

use serde::{Deserialize, Serialize};
use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::*;
use crate::channels::mutators::Mutator;

#[derive(Debug, Serialize, Deserialize)]
pub struct Join {
    pub field: Value,
    pub separator: String,
    pub new_field: Option<String>,
}

impl Mutator for Join {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        match &self.field {
            Value::String(s) => {
                let value = v.get(s.as_str())
                    .ok_or_else(|| Error::FieldNotFoundInJSON(self.field.to_string()))?;

                let array = match value {
                    Value::Array(ar) => ar,
                    _ => return Error::NotAnArray(self.field.to_string()).into()
                };

                let new_value = array.clone().into_iter()
                    .filter_map(|x| {
                        match x {
                            Value::String(s) => Some(s),
                            _ => None,
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(self.separator.as_str());

                v[s.as_str()] = Value::from(new_value);

                Ok(())
            }
            Value::Array(ar) => {
                let mut out: Vec<String> = Vec::new();
                let new_field = self.new_field.as_ref()
                    .ok_or_else(|| Error::RequiredFieldNotFound("new_field".to_string()))?;

                for value in ar {
                    let field_name = match value {
                        Value::String(s) => s,
                        _ => return Error::ValueInArrayIsNotString(value.to_string()).into(),
                    };

                    let field = v.get(field_name.as_str())
                        .ok_or_else(|| Error::FieldNotFoundInJSON(field_name.to_string()))?;

                    match field {
                        Value::String(s) => out.push(s.clone()),
                        _ => return Error::NotString(value.to_string()).into(),
                    };
                }

                let new_value = out.join(self.separator.as_str());
                v.insert(new_field.clone(), Value::from(new_value));

                Ok(())
            }
            _ => Error::JoinErrorTypeNotRecognized.into()
        }
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Join
    }
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Join field: '{}' using separator '{}'", self.field, self.separator)
    }
}
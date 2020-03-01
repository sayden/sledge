use std::fmt;
use std::fmt::Error;

use serde::{Deserialize, Serialize};
use serde::export::Formatter;
use serde_json::{Map, Value};

use crate::channels::core::*;
use crate::channels::core::Mutator;

#[derive(Debug, Serialize, Deserialize)]
pub struct Join {
    #[serde(rename = "type")]
    pub type_: String,
    pub field: Value,
    pub separator: String,
    pub new_field: String,
}

impl Mutator for Join {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        match &self.field {
            Value::String(s) => {
                let maybe_field = v.get(s.as_str());

                let field = match maybe_field {
                    None => return Some(anyhow!("value '{}' not found", self.field)),
                    Some(v) => v,
                };

                let array = match field {
                    Value::Array(ar) => ar,
                    _ => return Some(anyhow!("value '{}' is not an array", self.field))
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

                None
            }
            Value::Array(ar) => {
                let mut out: Vec<String> = Vec::new();

                for value in ar {
                    let field_name = match value {
                        Value::String(s) => s,
                        _ => return Some(anyhow!("value on array '{}' is not an string", value)),
                    };

                    let maybe_field = match v.get(field_name.as_str()) {
                        Some(f) => f,
                        None => return Some(anyhow!("field '{}' is not a value", field_name)),
                    };

                    match maybe_field {
                        Value::String(s) => out.push(s.clone()),
                        _ => return Some(anyhow!("field '{}' is not an string", maybe_field)),
                    };
                }

                let new_value = out.join(self.separator.as_str());
                v.insert((&self.new_field).clone(), Value::from(new_value));

                None
            }
            _ => None
        }
    }
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Join field: '{}' using separator '{}'", self.field, self.separator)
    }
}
use crate::processors::core::ModifierTrait;
use crate::processors::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Join {
    pub modifier: Modifier,
    pub separator: String,
}

impl ModifierTrait for Join {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_value = v.get(&self.modifier.field);

        let value = match maybe_value {
            None => return Some(anyhow!("value '{}' not found", self.modifier.field)),
            Some(v) => v,
        };

        let array = match value {
            Value::Array(ar) => ar,
            _ => return Some(anyhow!("value '{}' is not an array", self.modifier.field))
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

        v[self.modifier.field.as_str()] = Value::from(new_value);

        None
    }
}

impl fmt::Display for Join {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Join field: '{}' using separator '{}'", self.modifier.field, self.separator)
    }
}
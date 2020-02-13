use crate::processors::chain::ModifierTrait;
use crate::processors::chain::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Append {
    pub modifier: Modifier,
    pub append: String,
}

impl ModifierTrait for Append {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        let maybe_field = v.get(&self.modifier.field);

        self.exists(maybe_field,&self.modifier.field)?;

        match maybe_field {
            Some(value) => {
                let new_value = format!("{}{}", value.as_str()?, self.append);
                v[&self.modifier.field] = Value::from(new_value);
                None
            }
            None => Some(anyhow!("value '{}' not found", self.modifier.field)),
        }
    }
}

impl fmt::Display for Append {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Append '{}' to field: '{}'", self.append, self.modifier.field)
    }
}
use crate::processors::chain::ModifierTrait;
use crate::processors::chain::*;

use serde_json::{Map, Value};
use std::fmt;
use std::fmt::{Error, Formatter};

#[derive(Debug)]
pub struct Sort {
    pub modifier: Modifier,
    pub descending: bool,
}

impl ModifierTrait for Sort {
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

        let is_string = match array.get(0) {
            Some(v) => v.is_string(),
            _ => false,
        };

        if is_string {
            let mut result = array.into_iter()
                .filter_map(|x: &Value| match x {
                    Value::String(s) => Some(s),
                    _ => None,
                }).collect::<Vec<&String>>();

            result.sort();

            if self.descending {
                result.reverse();
            }

            let final_result = result.into_iter().map(move |x| Value::from(x.as_str())).collect::<Vec<Value>>();
            v[self.modifier.field.as_str()] = Value::from(final_result);
        } else {
            let mut result = array.into_iter()
                .filter_map(|x: &Value| match x {
                    Value::Number(n) => Some(n.as_i64().unwrap()),
                    _ => None
                }).collect::<Vec<i64>>();

            result.sort();

            if self.descending {
                result.reverse();
            }

            let final_result = result.into_iter().map(move |x| Value::from(x)).collect::<Vec<Value>>();
            v[self.modifier.field.as_str()] = Value::from(final_result);
        }

        None
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Sort '{}' field: (descending='{}')", self.modifier.field, self.descending)
    }
}
use std::fmt;
use std::fmt::Formatter;

use itertools::Itertools;
use serde_json::{Map, Value};

use crate::channels::error::Error;
use crate::channels::mutators::Mutator;
use crate::channels::mutators::*;

#[derive(Debug)]
pub struct Sort {
    pub modifier: Mutation,
    pub descending: bool,
}

fn maybe_reverse<T: Into<Value> + Ord>(
    descending: bool,
    res: impl Iterator<Item = T> + DoubleEndedIterator,
) -> Vec<Value> {
    if descending {
        res.sorted().rev().map(|x| x.into()).collect::<Vec<Value>>()
    } else {
        res.sorted().map(|x| x.into()).collect::<Vec<Value>>()
    }
}

impl Mutator for Sort {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error> {
        let value = v
            .get(&self.modifier.field)
            .ok_or_else(|| Error::FieldNotFoundInJSON(self.modifier.field.to_string()))?;

        let array = match value {
            Value::Array(ar) => ar,
            _ => return Error::NotAnArray(self.modifier.field.to_string()).into(),
        };

        let first_item = array
            .get(0)
            .ok_or_else(|| Error::EmptyArray(self.modifier.field.to_string()))?;

        match first_item {
            Value::String(_) => {
                let res = array.iter().filter_map(|x: &Value| match x {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                });

                let res = maybe_reverse(self.descending, res);
                v[self.modifier.field.as_str()] = Value::from(res);
            }
            Value::Number(_) => {
                let res = array.iter().filter_map(|x: &Value| match x {
                    Value::Number(n) => {
                        let maybe_i64 = n.as_i64();
                        if maybe_i64.is_none() {
                            log::error!("error trying to get an i64 value from json");
                        }
                        maybe_i64
                    }
                    _ => None,
                });

                let res = maybe_reverse(self.descending, res);
                v[self.modifier.field.as_str()] = Value::from(res);
            }
            _ => return Error::SortNotPossibleError.into(),
        }

        Ok(())
    }

    fn mutator_type(&self) -> MutatorType {
        MutatorType::Sort
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Sort '{}' field: (descending='{}')",
            self.modifier.field, self.descending
        )
    }
}

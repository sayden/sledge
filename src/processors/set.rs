use crate::processors::core::Modifier;
use crate::processors::core::*;

use serde_json::{Map, Value};
use std::fmt;
use serde::export::Formatter;
use std::fmt::Error;

#[derive(Debug)]
pub struct Set {
    pub modifier: Processor,
    pub value: Value,
}

impl Modifier for Set {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<anyhow::Error> {
        v.insert(self.modifier.field.clone(), self.value.clone());

        None
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Set '{}' to value: '{}'", self.modifier.field, self.value)
    }
}
use serde_json::Value;
use crate::channels::core::{factory, Mutator};
use anyhow::Error;
use std::ops::Deref;
use std::borrow::BorrowMut;

pub struct Processors(Vec<Box<dyn Mutator>>);

impl Processors {
    pub fn new(mo: String) -> Result<Self, Error> {
        let ms: Vec<Value> = serde_json::from_str(mo.as_str())
            .or_else(|err| bail!("error tyring to parse modifiers: {:?}", err))?;
        let modifiers = ms.into_iter()
            .filter_map(|x| factory(x))
            .collect::<Vec<Box<dyn Mutator>>>();

        Ok(Processors(modifiers))
    }
}

impl Deref for Processors {
    type Target = Vec<Box<dyn Mutator>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_and_modify(json_data: &str, mods: &Processors) -> Result<String, Error> {
    let mut p: Value = serde_json::from_str(json_data)
        .or_else(|err| bail!("error trying to parse incoming json {:?}", err))?;
    let mutp = p.as_object_mut()
        .ok_or(anyhow!("error trying to create mutable reference to json"))?;

    for modifier in mods.iter() {
        match modifier.mutate(mutp.borrow_mut()) {
            Some(err) => {
                warn!("error trying to modify json '{}'", err);
            }
            _ => (),
        }
    }

    serde_json::to_string(&mutp)
        .or_else(|err: serde_json::error::Error| bail!("error generating json string: '{:?}", err))
}
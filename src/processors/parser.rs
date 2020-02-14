use serde_json::{Value, Map};
use crate::processors::core::{factory, ModifierTrait};
use anyhow::Error;
use std::ops::Deref;
use std::borrow::{Borrow, BorrowMut};

pub struct Modifiers(Vec<Box<dyn ModifierTrait>>);

impl Modifiers {
    pub fn new(mo: String) -> Result<Self, Error> {
        let ms: Vec<Value> = serde_json::from_str(mo.as_str())?;
        let modifiers = ms.into_iter()
            .filter_map(|x| factory(x))
            .collect::<Vec<Box<dyn ModifierTrait>>>();

        Ok(Modifiers(modifiers))
    }
}

impl Deref for Modifiers {
    type Target = Vec<Box<dyn ModifierTrait>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse2(mut incoming: Map<String, Value>, mods: &Vec<Box<dyn ModifierTrait>>) -> Result<String, Error> {
    for modifier in mods.iter() {
        match modifier.modify(incoming.borrow_mut()) {
            Some(err) => {
                println!("error trying to modify json '{}'", err);
            }
            _ => (),
        }
    }

    serde_json::to_string(&incoming)
        .or_else(|err: serde_json::error::Error| Err(anyhow!("error generating json string")))
}
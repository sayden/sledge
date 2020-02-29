use serde_json::Value;
use crate::channels::core::{factory, Mutator};
use anyhow::Error;
use std::ops::Deref;
use std::borrow::BorrowMut;
use serde::{Serialize,Deserialize};

pub struct Channel(Vec<Box<dyn Mutator>>);

#[derive(Serialize,Deserialize)]
pub struct NewChannel {
    name: String,
    channel: Vec<Value>,
}

impl Channel {
    pub fn new(mo: &str) -> Result<Self, Error> {
        let ms: NewChannel = serde_json::from_str(mo)
            .or_else(|err| bail!("error tyring to parse modifiers: {:?}", err))?;

        let mutators = ms.channel.into_iter()
            .filter_map(|x| factory(x))
            .collect::<Vec<Box<dyn Mutator>>>();

        Ok(Channel(mutators))
    }
}

impl Deref for Channel {
    type Target = Vec<Box<dyn Mutator>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_and_modify(json_data: &str, mods: &Channel) -> Result<String, Error> {
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
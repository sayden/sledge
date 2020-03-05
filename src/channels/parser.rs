use std::borrow::BorrowMut;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::channels::core::{factory, Mutator, MutatorType};
use crate::components::storage::Error;

pub struct Channel {
    pub name: String,
    pub channel: Vec<Box<dyn Mutator>>,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelToParseJSON {
    name: String,
    channel: Vec<Value>,
}

impl Channel {
    pub fn new_str(mo: &str) -> Result<Self, Error> {
        let ms: ChannelToParseJSON = serde_json::from_str(mo)
            .or_else(|err| Err(Error::Serialization(err)))?;

        Channel::new(ms)
    }

    pub fn new_vec(mo: Vec<u8>) -> Result<Self, Error<'a>> {
        let ms: ChannelToParseJSON = serde_json::from_slice(mo.as_slice())
            .or_else(|err| Err(Error::Serialization(err)))?;

        Channel::new(ms)
    }

    pub fn new_u8(mo: &[u8]) -> Result<Self, Error> {
        let ms: ChannelToParseJSON = serde_json::from_slice(mo)
            .or_else(|err| Err(Error::Serialization(err)))?;

        Channel::new(ms)
    }

    fn new<'a>(ms: ChannelToParseJSON) -> Result<Self, Error<'a>> {
        let mutators = ms.channel.into_iter()
            .filter_map(|x| factory(x.clone())
                .or_else(|| {
                    log::error!("channel parsing error {}", x);
                    None
                })
            )
            .collect::<Vec<Box<dyn Mutator>>>();

        Ok(Channel { name: "".to_string(), channel: mutators })
    }
}

impl Deref for Channel {
    type Target = Vec<Box<dyn Mutator>>;
    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

pub fn parse_and_modify_u8<'a>(input_data: &[u8], mods: &'a Channel) -> Result<Vec<u8>, Error<'a>> {
    if mods.len() == 0 {
        return Err(Error::EmptyMutator);
    }

    let first_mod = mods.first().unwrap();
    let maybe_value = match first_mod.mutator_type() {
        MutatorType::Grok => {
            let g = first_mod.as_grok().unwrap();
            if g.modifier.field == "_plain_input" {
                g.mutate_plain_string(input_data)
            } else {
                None
            }
        }
        _ => None,
    };


    match maybe_value {
        Some(x) => parse_and_modify(x, mods),
        None => serde_json::from_slice(input_data)
            .or_else(|err| Err(Error::Serialization(err)))
            .and_then(|x| parse_and_modify(x, mods)),
    }
}

fn parse_and_modify(mut p: Value, mods: &Channel) -> Result<Vec<u8>, Error> {
    let mutp = p.as_object_mut()
        .ok_or(Error::SerializationString("error trying to create mutable reference to json".to_string()))?;

    for modifier in mods.iter() {
        match modifier.mutate(mutp.borrow_mut()) {
            Some(err) => {
                log::warn!("error trying to modify json '{}'", err);
            }
            _ => (),
        }
    }

    serde_json::to_vec(&mutp)
        .or_else(|err: serde_json::error::Error| Err(Error::Serialization(err)))
}
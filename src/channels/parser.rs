use serde_json::Value;
use crate::channels::core::{factory, Mutator};
use std::ops::Deref;
use std::borrow::BorrowMut;
use serde::{Serialize, Deserialize};
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
    pub fn new(mo: &str) -> Result<Self, Error> {
        let ms: ChannelToParseJSON = serde_json::from_str(mo).or_else(|err| Err(Error::Serialization(err)))?;

        let mutators = ms.channel.into_iter()
            .filter_map(|x| factory(x))
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

pub fn parse_and_modify_str(json_data: &str, mods: &Channel) -> Result<Vec<u8>, Error> {
    let p: Value = serde_json::from_str(json_data).or_else(|err| Err(Error::Serialization(err)))?;
    parse_and_modify(p, mods)
}

pub fn parse_and_modify_u8(json_data: &[u8], mods: &Channel) -> Result<Vec<u8>, Error> {
    let p: Value = serde_json::from_slice(json_data).or_else(|err| Err(Error::Serialization(err)))?;
    parse_and_modify(p, mods)
}

fn parse_and_modify(mut p: Value, mods: &Channel) -> Result<Vec<u8>, Error> {
    let mutp = p.as_object_mut()
        .ok_or(Error::SerializationString("error trying to create mutable reference to json".to_string()))?;

    for modifier in mods.iter() {
        match modifier.mutate(mutp.borrow_mut()) {
            Some(err) => {
                warn!("error trying to modify json '{}'", err);
            }
            _ => (),
        }
    }

    serde_json::to_vec(&mutp)
        .or_else(|err: serde_json::error::Error| Err(Error::Serialization(err)))
}
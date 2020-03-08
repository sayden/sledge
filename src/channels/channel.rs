use std::borrow::BorrowMut;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::channels::mutators::{factory, Mutator, MutatorType};
use crate::components::errors::Error;

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
            .or_else(|err| Err(Error::SerdeError(err)))?;

        Channel::new(ms)
    }

    pub fn new_vec(mo: Vec<u8>) -> Result<Self, Error> {
        let ms: ChannelToParseJSON = serde_json::from_slice(mo.as_slice())
            .or_else(|err| Err(Error::SerdeError(err)))?;

        Channel::new(ms)
    }

    pub fn new_u8(mo: &[u8]) -> Result<Self, Error> {
        let ms: ChannelToParseJSON = serde_json::from_slice(mo)
            .or_else(|err| Err(Error::SerdeError(err)))?;

        Channel::new(ms)
    }

    fn new(ms: ChannelToParseJSON) -> Result<Self, Error> {
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

pub fn parse_and_modify_u8<'a>(input_data: &[u8], mods: &Channel, omit_errors: bool) -> Option<Vec<u8>> {
    if mods.len() == 0 {
        log::warn!("mutator list in channel is empty");
        return None;
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

    let x = maybe_value.or(serde_json::from_slice(input_data)
        .map_err(|err| log::warn!("error trying to mutate value: {}", err))
        .ok())?;

    parse_and_modify(x, mods, omit_errors)
}

fn parse_and_modify(mut p: Value, mods: &Channel, omit_errors: bool) -> Option<Vec<u8>> {
    let mutp = p.as_object_mut()?;

    for modifier in mods.iter() {
        match modifier.mutate(mutp.borrow_mut()) {
            Some(err) => {
                log::warn!("error trying to modify json '{}'", err);
                if omit_errors {
                    return None;
                }
            }
            _ => (),
        }
    }

    serde_json::to_vec(&mutp)
        .map_err(|err| log::warn!("error trying to create mutable reference to json: {}", err.to_string()))
        .ok()
}
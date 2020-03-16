use std::borrow::BorrowMut;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::channels::mutators::{factory, Mutator, MutatorType};
use crate::components::errors::Error;

pub struct Channel {
    pub name: String,
    pub channel: Vec<Box<dyn Mutator>>,
    pub omit_errors: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelToParseJSON {
    name: String,
    channel: Vec<Value>,
}

impl Channel {
    pub fn new_vec(mo: Vec<u8>, omit_errors: bool) -> Result<Self, Error> {
        let ms: ChannelToParseJSON =
            serde_json::from_slice(mo.as_slice()).or_else(|err| Err(Error::SerdeError(err)))?;

        Channel::new(ms, omit_errors)
    }

    fn new(ms: ChannelToParseJSON, omit_errors: bool) -> Result<Self, Error> {
        let mutators = ms
            .channel
            .into_iter()
            .filter_map(|x| {
                match factory(x.clone()) {
                    Err(e) => {
                        log::error!("channel parsing error {}. Error: {}", x, e.to_string());
                        None
                    }
                    Ok(v) => Some(v),
                }
            })
            .collect::<Vec<Box<dyn Mutator>>>();

        Ok(Channel {
            name: "".to_string(),
            channel: mutators,
            omit_errors,
        })
    }

    pub fn parse_and_modify<'a>(&self, input_data: &[u8]) -> Option<Vec<u8>> {
        if self.channel.len() == 0 {
            log::warn!("mutator list in channel is empty");
            return None;
        }

        let first_mod = self.channel.first().unwrap();
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

        let mut x = maybe_value.or(serde_json::from_slice(input_data)
            .map_err(|err| log::warn!("error trying to mutate value: {}", err))
            .ok())?;

        let mutp = x.as_object_mut()?;

        for modifier in self.channel.iter() {
            match modifier.mutate(mutp.borrow_mut()) {
                Err(err) => {
                    log::warn!("error trying to modify json '{}'", err);
                    if self.omit_errors {
                        return None;
                    }
                }
                _ => (),
            }
        }

        serde_json::to_vec(&mutp)
            .map_err(|err| {
                log::warn!(
                    "error trying to create mutable reference to json: {}",
                    err.to_string()
                )
            })
            .ok()
    }
}

impl Deref for Channel {
    type Target = Vec<Box<dyn Mutator>>;
    fn deref(&self) -> &Self::Target { &self.channel }
}

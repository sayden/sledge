use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use anyhow::Error;
use crate::processors::remove::Remove;
use crate::processors::append::Append;
use crate::processors::rename::Rename;
use crate::processors::join::Join;
use crate::processors::upper_lower_case::UpperLowercase;
use crate::processors::set::Set;
use crate::processors::split::Split;
use crate::processors::trim_spaces::TrimSpaces;
use crate::processors::trim::Trim;
use crate::processors::sort::Sort;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct Modifier {
    #[serde(rename = "type")]
    pub type_: String,
    pub field: String,
}

pub trait ModifierTrait {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<Error>;
}


#[derive(Debug)]
pub enum Modifiers {
    Append,
    Join,
    Lowercase,
    Remove,
    Rename,
    Set,
    Sort,
    Split,
    Trim,
    TrimSpace,
    Uppercase,
}

impl FromStr for Modifiers {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "append" => Ok(Modifiers::Append),
            "join" => Ok(Modifiers::Join),
            "lowercase" => Ok(Modifiers::Lowercase),
            "remove" => Ok(Modifiers::Remove),
            "rename" => Ok(Modifiers::Rename),
            "set" => Ok(Modifiers::Set),
            "sort" => Ok(Modifiers::Sort),
            "split" => Ok(Modifiers::Split),
            "trim" => Ok(Modifiers::Trim),
            "trim_space" => Ok(Modifiers::TrimSpace),
            "uppercase" => Ok(Modifiers::Uppercase),
            _ => Err(())
        }
    }
}

impl ToString for Modifiers {
    fn to_string(&self) -> String {
        match self {
            Modifiers::Append => "append".to_string(),
            Modifiers::Join => "join".to_string(),
            Modifiers::Lowercase => "lowercase".to_string(),
            Modifiers::Remove => "remove".to_string(),
            Modifiers::Rename => "rename".to_string(),
            Modifiers::Set => "set".to_string(),
            Modifiers::Sort => "sort".to_string(),
            Modifiers::Split => "split".to_string(),
            Modifiers::Trim => "trim".to_string(),
            Modifiers::TrimSpace => "trim_space".to_string(),
            Modifiers::Uppercase => "uppercase".to_string(),
        }
    }
}

pub fn factory(v: &Value) -> Option<Box<dyn ModifierTrait>> {
    let type_: Modifiers = v["type"].as_str()?.parse().ok()?;
    match type_ {
        Modifiers::Remove =>
            Some(Box::new(Remove {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                }
            })),
        Modifiers::Append => Some(Box::new(Append {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            append: v["append"].as_str()?.to_string(),
        })),
        Modifiers::Rename => Some(Box::new(Rename {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            rename: v["new_name"].as_str()?.to_string(),
        })),
        Modifiers::Join => Some(Box::new(Join {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            separator: v["separator"].as_str()?.to_string(),
        })),
        Modifiers::Lowercase => Some(Box::new(
            UpperLowercase {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_lowercase,
            })),
        Modifiers::Uppercase => Some(Box::new(
            UpperLowercase {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_uppercase,
            })),
        Modifiers::Split => Some(Box::new(
            Split {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                separator: v["separator"].as_str()?.to_string(),
            })),
        Modifiers::TrimSpace => Some(Box::new(
            TrimSpaces {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
            })),
        Modifiers::Trim => Some(Box::new(
            Trim {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                from: v["from"].as_str()?.to_string(),
                total: v["total"].as_i64()? as usize,
            })),
        Modifiers::Set =>
            Some(Box::new(
                Set {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    value: v["value"].clone(),
                })),
        Modifiers::Sort =>
            Some(Box::new(
                Sort {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    descending: v["descending"].as_bool()?.clone(),
                })),
        _ => {
            println!("Modifier with type '{}' not found", v["type"].as_str()?);
            None
        }
    }
}
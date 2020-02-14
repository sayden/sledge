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
pub enum ModifierType {
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

impl FromStr for ModifierType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "append" => Ok(ModifierType::Append),
            "join" => Ok(ModifierType::Join),
            "lowercase" => Ok(ModifierType::Lowercase),
            "remove" => Ok(ModifierType::Remove),
            "rename" => Ok(ModifierType::Rename),
            "set" => Ok(ModifierType::Set),
            "sort" => Ok(ModifierType::Sort),
            "split" => Ok(ModifierType::Split),
            "trim" => Ok(ModifierType::Trim),
            "trim_space" => Ok(ModifierType::TrimSpace),
            "uppercase" => Ok(ModifierType::Uppercase),
            _ => Err(())
        }
    }
}

impl ToString for ModifierType {
    fn to_string(&self) -> String {
        match self {
            ModifierType::Append => "append".to_string(),
            ModifierType::Join => "join".to_string(),
            ModifierType::Lowercase => "lowercase".to_string(),
            ModifierType::Remove => "remove".to_string(),
            ModifierType::Rename => "rename".to_string(),
            ModifierType::Set => "set".to_string(),
            ModifierType::Sort => "sort".to_string(),
            ModifierType::Split => "split".to_string(),
            ModifierType::Trim => "trim".to_string(),
            ModifierType::TrimSpace => "trim_space".to_string(),
            ModifierType::Uppercase => "uppercase".to_string(),
        }
    }
}

pub fn factory(v: Value) -> Option<Box<dyn ModifierTrait>> {
    let type_: ModifierType = v["type"].as_str()?.parse().ok()?;

    match type_ {
        ModifierType::Remove =>
            Some(Box::new(Remove {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                }
            })),
        ModifierType::Append => Some(Box::new(Append {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            append: v["append"].as_str()?.to_string(),
        })),
        ModifierType::Rename => Some(Box::new(Rename {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            rename: v["new_name"].as_str()?.to_string(),
        })),
        ModifierType::Join => Some(Box::new(Join {
            modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            separator: v["separator"].as_str()?.to_string(),
        })),
        ModifierType::Lowercase => Some(Box::new(
            UpperLowercase {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_lowercase,
            })),
        ModifierType::Uppercase => Some(Box::new(
            UpperLowercase {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_uppercase,
            })),
        ModifierType::Split => Some(Box::new(
            Split {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                separator: v["separator"].as_str()?.to_string(),
            })),
        ModifierType::TrimSpace => Some(Box::new(
            TrimSpaces {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
            })),
        ModifierType::Trim => Some(Box::new(
            Trim {
                modifier: Modifier {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                from: v["from"].as_str()?.to_string(),
                total: v["total"].as_i64()? as usize,
            })),
        ModifierType::Set =>
            Some(Box::new(
                Set {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    value: v["value"].clone(),
                })),
        ModifierType::Sort =>
            Some(Box::new(
                Sort {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    descending: v["descending"].as_bool()?.clone(),
                })),
    }
}
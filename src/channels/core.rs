use std::str::FromStr;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::channels::append::Append;
use crate::channels::grok::Grok_;
use crate::channels::join::Join;
use crate::channels::remove::Remove;
use crate::channels::rename::Rename;
use crate::channels::set::Set;
use crate::channels::sort::Sort;
use crate::channels::split::Split;
use crate::channels::trim::Trim;
use crate::channels::trim_spaces::TrimSpaces;
use crate::channels::upper_lower_case::UpperLowercase;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mutation {
    #[serde(rename = "type")]
    pub type_: String,
    pub field: String,
}

pub trait Mutator: Send + Sync {
    fn mutate(&self, v: &mut Map<String, Value>) -> Option<Error>;
}


#[derive(Debug)]
pub enum MutatorType {
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
    Grok,
}

impl FromStr for MutatorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "append" => Ok(MutatorType::Append),
            "join" => Ok(MutatorType::Join),
            "lowercase" => Ok(MutatorType::Lowercase),
            "remove" => Ok(MutatorType::Remove),
            "rename" => Ok(MutatorType::Rename),
            "set" => Ok(MutatorType::Set),
            "sort" => Ok(MutatorType::Sort),
            "split" => Ok(MutatorType::Split),
            "trim" => Ok(MutatorType::Trim),
            "trim_space" => Ok(MutatorType::TrimSpace),
            "uppercase" => Ok(MutatorType::Uppercase),
            "grok" => Ok(MutatorType::Grok),
            _ => Err(())
        }
    }
}

impl ToString for MutatorType {
    fn to_string(&self) -> String {
        match self {
            MutatorType::Append => "append".to_string(),
            MutatorType::Join => "join".to_string(),
            MutatorType::Lowercase => "lowercase".to_string(),
            MutatorType::Remove => "remove".to_string(),
            MutatorType::Rename => "rename".to_string(),
            MutatorType::Set => "set".to_string(),
            MutatorType::Sort => "sort".to_string(),
            MutatorType::Split => "split".to_string(),
            MutatorType::Trim => "trim".to_string(),
            MutatorType::TrimSpace => "trim_space".to_string(),
            MutatorType::Uppercase => "uppercase".to_string(),
            MutatorType::Grok => "grok".to_string(),
        }
    }
}

pub fn factory(v: Value) -> Option<Box<dyn Mutator>> {
    let type_: MutatorType = v["type"].as_str()?.parse().ok()?;

    match type_ {
        MutatorType::Remove =>
            Some(Box::new(Remove {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                }
            })),
        MutatorType::Append => Some(Box::new(Append {
            modifier: Mutation { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            append: v["append"].as_str()?.to_string(),
        })),
        MutatorType::Rename => Some(Box::new(Rename {
            modifier: Mutation { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            rename: v["new_name"].as_str()?.to_string(),
        })),
        MutatorType::Join => Some(Box::new(Join {
            type_: type_.to_string(),
            field: v["field"].clone(),
            separator: v["separator"].as_str()?.to_string(),
            new_field: v["new_field"].as_str().unwrap_or_default().to_string(),
        })),
        MutatorType::Lowercase => Some(Box::new(
            UpperLowercase {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_lowercase,
            })),
        MutatorType::Uppercase => Some(Box::new(
            UpperLowercase {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_uppercase,
            })),
        MutatorType::Split => Some(Box::new(
            Split {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                separator: v["separator"].as_str()?.to_string(),
            })),
        MutatorType::TrimSpace => Some(Box::new(
            TrimSpaces {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
            })),
        MutatorType::Trim => Some(Box::new(
            Trim {
                modifier: Mutation {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                from: v["from"].as_str()?.to_string(),
                total: v["total"].as_i64()? as usize,
            })),
        MutatorType::Set =>
            Some(Box::new(
                Set {
                    modifier: Mutation {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    value: v["value"].clone(),
                })),
        MutatorType::Sort =>
            Some(Box::new(
                Sort {
                    modifier: Mutation {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    descending: v["descending"].as_bool()?.clone(),
                })),
        MutatorType::Grok =>
            Some(Box::new(
                Grok_ {
                    modifier: Mutation {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    // custom_patterns: v["custom_patterns"].clone().as_array(),
                    pattern: v["pattern"].as_str()?.to_string(),
                })),
    }
}
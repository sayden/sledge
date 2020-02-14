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
pub struct Processor {
    #[serde(rename = "type")]
    pub type_: String,
    pub field: String,
}

pub trait Modifier {
    fn modify(&self, v: &mut Map<String, Value>) -> Option<Error>;
}


#[derive(Debug)]
pub enum ProcessorType {
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

impl FromStr for ProcessorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "append" => Ok(ProcessorType::Append),
            "join" => Ok(ProcessorType::Join),
            "lowercase" => Ok(ProcessorType::Lowercase),
            "remove" => Ok(ProcessorType::Remove),
            "rename" => Ok(ProcessorType::Rename),
            "set" => Ok(ProcessorType::Set),
            "sort" => Ok(ProcessorType::Sort),
            "split" => Ok(ProcessorType::Split),
            "trim" => Ok(ProcessorType::Trim),
            "trim_space" => Ok(ProcessorType::TrimSpace),
            "uppercase" => Ok(ProcessorType::Uppercase),
            _ => Err(())
        }
    }
}

impl ToString for ProcessorType {
    fn to_string(&self) -> String {
        match self {
            ProcessorType::Append => "append".to_string(),
            ProcessorType::Join => "join".to_string(),
            ProcessorType::Lowercase => "lowercase".to_string(),
            ProcessorType::Remove => "remove".to_string(),
            ProcessorType::Rename => "rename".to_string(),
            ProcessorType::Set => "set".to_string(),
            ProcessorType::Sort => "sort".to_string(),
            ProcessorType::Split => "split".to_string(),
            ProcessorType::Trim => "trim".to_string(),
            ProcessorType::TrimSpace => "trim_space".to_string(),
            ProcessorType::Uppercase => "uppercase".to_string(),
        }
    }
}

pub fn factory(v: Value) -> Option<Box<dyn Modifier>> {
    let type_: ProcessorType = v["type"].as_str()?.parse().ok()?;

    match type_ {
        ProcessorType::Remove =>
            Some(Box::new(Remove {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                }
            })),
        ProcessorType::Append => Some(Box::new(Append {
            modifier: Processor { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            append: v["append"].as_str()?.to_string(),
        })),
        ProcessorType::Rename => Some(Box::new(Rename {
            modifier: Processor { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            rename: v["new_name"].as_str()?.to_string(),
        })),
        ProcessorType::Join => Some(Box::new(Join {
            modifier: Processor { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
            separator: v["separator"].as_str()?.to_string(),
        })),
        ProcessorType::Lowercase => Some(Box::new(
            UpperLowercase {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_lowercase,
            })),
        ProcessorType::Uppercase => Some(Box::new(
            UpperLowercase {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                f: str::to_uppercase,
            })),
        ProcessorType::Split => Some(Box::new(
            Split {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                separator: v["separator"].as_str()?.to_string(),
            })),
        ProcessorType::TrimSpace => Some(Box::new(
            TrimSpaces {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
            })),
        ProcessorType::Trim => Some(Box::new(
            Trim {
                modifier: Processor {
                    type_: type_.to_string(),
                    field: v["field"].as_str()?.to_string(),
                },
                from: v["from"].as_str()?.to_string(),
                total: v["total"].as_i64()? as usize,
            })),
        ProcessorType::Set =>
            Some(Box::new(
                Set {
                    modifier: Processor {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    value: v["value"].clone(),
                })),
        ProcessorType::Sort =>
            Some(Box::new(
                Sort {
                    modifier: Processor {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    descending: v["descending"].as_bool()?.clone(),
                })),
    }
}
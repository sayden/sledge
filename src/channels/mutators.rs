use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::channels::append::Append;
use crate::channels::error::Error;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mutation {
    pub field: String,
}

pub trait Mutator: Send + Sync {
    fn mutate(&self, v: &mut Map<String, Value>) -> Result<(), Error>;
    fn mutator_type(&self) -> MutatorType;
    fn as_grok(&self) -> Option<&Grok_> { None }
}

#[derive(Debug)]
pub enum MutatorType {
    Append,
    Grok,
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
            _ => Err(()),
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

pub fn factory(v: Value) -> Result<Box<dyn Mutator>, Error> {
    let type_: MutatorType = v["type"]
        .as_str()
        .ok_or_else(|| Error::KeyNotFoundInMutator("type".to_string()))?
        .parse()
        .map_err(|_| Error::ParsingError("type".to_string()))?;

    if type_.to_string() == MutatorType::Join.to_string() {
        return Ok(box (Join {
            field: v["field"].clone(),
            separator: v["separator"]
                .as_str()
                .ok_or_else(|| Error::KeyNotFoundInMutator("separator".to_string()))?
                .to_string(),
            new_field: v["new_field"].as_str().map(|x| x.to_string()),
        }));
    }

    let field = v["field"]
        .as_str()
        .ok_or_else(|| Error::KeyNotFoundInMutator("field".to_string()))?
        .to_string();

    match type_ {
        MutatorType::Remove => {
            Ok(box (Remove {
                modifier: Mutation { field },
            }))
        }
        MutatorType::Append => {
            Ok(box (Append {
                modifier: Mutation { field },
                append: v["append"]
                    .as_str()
                    .ok_or_else(|| Error::NotString("append".to_string()))?
                    .to_string(),
            }))
        }
        MutatorType::Rename => {
            Ok(box (Rename {
                modifier: Mutation { field },
                rename: v["new_name"]
                    .as_str()
                    .ok_or_else(|| Error::NotString("new_name".to_string()))?
                    .to_string(),
            }))
        }
        MutatorType::Lowercase => {
            Ok(box (UpperLowercase {
                modifier: Mutation { field },
                f: str::to_lowercase,
            }))
        }
        MutatorType::Uppercase => {
            Ok(box (UpperLowercase {
                modifier: Mutation { field },
                f: str::to_uppercase,
            }))
        }
        MutatorType::Split => {
            Ok(box (Split {
                modifier: Mutation { field },
                separator: v["separator"]
                    .as_str()
                    .ok_or_else(|| Error::NotString("separator".to_string()))?
                    .to_string(),
            }))
        }
        MutatorType::TrimSpace => {
            Ok(box (TrimSpaces {
                modifier: Mutation { field },
            }))
        }
        MutatorType::Trim => {
            Ok(box (Trim {
                modifier: Mutation { field },
                from: v["from"]
                    .as_str()
                    .ok_or_else(|| Error::NotString("from".to_string()))?
                    .to_string(),
                total: v["total"]
                    .as_i64()
                    .ok_or_else(|| Error::NotI64("total".to_string()))?
                    as usize,
            }))
        }
        MutatorType::Set => {
            Ok(box (Set {
                modifier: Mutation { field },
                value: v["value"].clone(),
            }))
        }
        MutatorType::Sort => {
            Ok(box (Sort {
                modifier: Mutation { field },
                descending: v["descending"]
                    .as_bool()
                    .ok_or_else(|| Error::NotBool("descending".to_string()))?,
            }))
        }
        MutatorType::Grok => {
            Ok(box (Grok_::new(
                field,
                v["pattern"]
                    .as_str()
                    .ok_or_else(|| Error::NotString("pattern".to_string()))?
                    .to_string(),
                v["custom_patterns"].as_array(),
            )
            .ok_or_else(|| Error::NotAnArray("custom_patterns".to_string()))?))
        }
        s => Err(Error::MutatorNotFound(s.to_string())),
    }
}

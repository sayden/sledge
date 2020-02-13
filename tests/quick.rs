#[cfg(test)]
mod quick {
    use serde_json::{Value, Map};
    use sledge::processors::chain::{Modifier, ModifierTrait};
    use sledge::processors::remove::*;
    use sledge::processors::append::Append;
    use sledge::processors::rename::Rename;
    use sledge::processors::join::Join;
    use sledge::processors::upper_lower_case::UpperLowercase;
    use sledge::processors::set::Set;
    use sledge::processors::split::Split;
    use sledge::processors::trim_spaces::TrimSpaces;
    use sledge::processors::trim::Trim;
    use sledge::processors::sort::Sort;

    #[test]
    fn test_after_key() {
        let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

        let mut v: Value = serde_json::from_str(data).unwrap();
        println!("Please call {} at the number ", v);

        let tree: &mut Map<String, Value> = v.as_object_mut().unwrap();
        let del_keys = vec!["age"];

        for (key, _) in tree.iter() {
            println!("{:?}", key);
        }

        for key in del_keys.iter() {
            tree.remove(&mut key.to_string());
        }

        for (key, _) in tree.iter() {
            println!("{:?}", key);
        }
    }

    #[test]
    fn test_remove_modifier() {
        let data = r#"{
                    "name": "John Doe",
                    "age": 43,
                    "delete":"this",
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ],
                    "phones_uk": [
                      "+44 1234567",
                      "+44 2345678"
                    ],
                    "to_sort": [4,3,8],
                    "to_sort_s": ["were", "asdasd", "qweqw"]
                  }"#;

        let mo = r#"[
            {
                "type": "remove",
                "field": "delete"
            },
            {
                "type": "append",
                "field": "name",
                "append": " hello   "
            },
            {
                "type": "rename",
                "field": "name",
                "new_name": "name_hello"
            },
            {
                "type": "join",
                "field": "phones_uk",
                "separator": ","
            },
            {
                "type": "lowercase",
                "field": "name_hello"
            },
            {
                "type": "set",
                "field": "my_field",
                "value": "my_value"
            },
            {
                "type": "trim_space",
                "field": "name_hello"
            },
            {
                "type": "split",
                "field": "name_hello",
                "separator": " "
            },
            {
                "type": "trim",
                "field": "my_field",
                "from": "right",
                "total": 2
            },
            {
                "type": "sort",
                "field": "to_sort_s",
                "descending": false
            },
            {
                "type": "sort",
                "field": "to_sort",
                "descending": true
            }
        ]"#;

        let mut p: Value = serde_json::from_str(data).unwrap();
        let mutp = p.as_object_mut().unwrap();
        let ms: Vec<Value> = serde_json::from_str(mo).unwrap();
        let modifiers = ms.into_iter()
            .filter_map(|x| factory(&x));

        let res = modifiers
            .fold(mutp, |acc, x| apply_modifier(acc, x));

        println!("{}", serde_json::to_string_pretty(res).unwrap());
    }

    fn factory(v: &Value) -> Option<Box<dyn ModifierTrait>> {
        let type_ = v["type"].as_str()?;
        match type_ {
            "remove" =>
                Some(Box::new(Remove {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    }
                })),
            "append" => Some(Box::new(Append {
                modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                append: v["append"].as_str()?.to_string(),
            })),
            "rename" => Some(Box::new(Rename {
                modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                rename: v["new_name"].as_str()?.to_string(),
            })),
            "join" => Some(Box::new(Join {
                modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                separator: v["separator"].as_str()?.to_string(),
            })),
            "lowercase" => Some(Box::new(
                UpperLowercase {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    f: str::to_lowercase,
                })),
            "uppercase" => Some(Box::new(
                UpperLowercase {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    f: str::to_uppercase,
                })),
            "split" => Some(Box::new(
                Split {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    separator: v["separator"].as_str()?.to_string(),
                })),
            "trim_space" => Some(Box::new(
                TrimSpaces {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                })),
            "trim" => Some(Box::new(
                Trim {
                    modifier: Modifier {
                        type_: type_.to_string(),
                        field: v["field"].as_str()?.to_string(),
                    },
                    from: v["from"].as_str()?.to_string(),
                    total: v["total"].as_i64()? as usize,
                })),
            "set" =>
                Some(Box::new(
                    Set {
                        modifier: Modifier {
                            type_: type_.to_string(),
                            field: v["field"].as_str()?.to_string(),
                        },
                        value: v["value"].clone(),
                    })),
            "sort" =>
                Some(Box::new(
                    Sort {
                        modifier: Modifier {
                            type_: type_.to_string(),
                            field: v["field"].as_str()?.to_string(),
                        },
                        descending: v["descending"].as_bool()?.clone(),
                    })),
            a => {
                println!("Modifier with type '{}' not found", a);
                None
            }
        }
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

    fn apply_modifier(acc: &mut Map<String, Value>, x: Box<dyn ModifierTrait>) -> &mut Map<String, Value> {
        match x.modify(acc) {
            None => acc,
            Some(err) => {
                println!("error trying to modify json '{}'", err);
                acc
            }
        }
    }
}
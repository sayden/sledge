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
                    "phones": [
                      "+44 1234567",
                      "+44 2345678"
                    ],
                    "phones_uk": [
                      "+44 1234567",
                      "+44 2345678"
                    ]
                  }"#;

        let mo = r#"[{
                    "type": "remove",
                    "field": "phones"
                  },{
                    "type": "remove",
                    "field": "age"
                  },{
                    "type": "append",
                    "field": "name",
                    "append": " hello"
                  },{
                    "type": "rename",
                    "field": "name",
                    "new_name": "name_hello"
                  },{
                    "type": "join",
                    "field": "phones_uk",
                    "separator": ","
                  },{
                    "type": "lowercase",
                    "field": "name_hello"
                  },{
                    "type": "set",
                    "field": "my_field",
                    "value": "my_value"
                  }]"#;

        let mut p: Value = serde_json::from_str(data).unwrap();
        let mutp = p.as_object_mut().unwrap();
        let ms: Vec<Value> = serde_json::from_str(mo).unwrap();
        let modifiers = ms.into_iter()
            .filter_map(|x| factory(&x));

        let res = modifiers
            .fold(mutp, |acc, x| apply_modifier(acc, x));

        println!("{}", serde_json::to_string_pretty(res).unwrap());

        let s = qwerq(str::to_lowercase, "Hello");
        println!("{}", s)

//        for x in  {
//            println!("{:?}", x.unwrap().modify(mutp).unwrap())
//        }
    }

    fn qwerq(f: fn(&str) -> String, s: &str) -> String {
        f(s)
//        str::to_lowercase("").to_string()
    }

    fn factory(v: &Value) -> Option<Box<dyn ModifierTrait>> {
        let type_ = v["type"].as_str()?;
        match type_ {
            "remove" => {
                let modifier = Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() };
                Some(Box::new(Remove { modifier }))
            }
            "append" => {
                Some(Box::new(Append {
                    modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                    append: v["append"].as_str()?.to_string(),
                }))
            }
            "rename" => {
                Some(Box::new(Rename {
                    modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                    rename: v["new_name"].as_str()?.to_string(),
                }))
            }
            "join" => {
                Some(Box::new(Join {
                    modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() },
                    separator: v["separator"].as_str()?.to_string(),
                }))
            }
            "lowercase" => {
                Some(Box::new(
                    UpperLowercase { modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() }, f: str::to_lowercase }))
            }
            "uppercase" => {
                Some(Box::new(
                    UpperLowercase { modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() }, f: str::to_uppercase }))
            }
            "set" => {
                Some(Box::new(
                    Set { modifier: Modifier { type_: type_.to_string(), field: v["field"].as_str()?.to_string() }, value: v["value"].clone() }))
            }
            a => {
                println!("Modifier with type '{}' not found", a);
                None
            }
        }
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
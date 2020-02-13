#[cfg(test)]
mod quick {
    use serde_json::{Value, Map};
    use sledge::processors::core::{ModifierTrait, factory};
    use sledge::processors::remove::Remove;
    use sledge::processors::rename::Rename;

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
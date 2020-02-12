#[cfg(test)]
mod quick {
    use serde_json::Value;

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

        let mut tree = v.as_object_mut().unwrap();
        let mut del_keys = vec!["age"];

        for (key, value) in tree.iter(){
            println!("{:?}", key);
        }

        for key in del_keys.iter(){
            tree.remove(&mut key.to_string());
        }

        for (key, value) in tree.iter(){
            println!("{:?}", key);
        }
    }

}
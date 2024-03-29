#[test]
fn test_parser() {
    use crate::channels::channel::parse_and_modify;
    #[cfg(test)]
    use crate::channels::channel::Channel;
    env_logger::init();

    let data = r#"{"name":"John", "surname": "Doe", "age":43,"delete":"this","phones":["+44 1234567","+44 2345678"],"phones_uk":["+44 1234567","+44 2345678"],"to_sort":[4,3,8],"to_sort_s":["were","asdasd","qweqw"]                  }"#;

    let mutators_json_array = r#"{
        "name": "my_channel",
        "channel": [
            {
                "type": "remove",
                "field": "delete"
            },
            {
                "type": "join",
                "field": ["name", "surname"],
                "separator": " ",
                "new_field": "full_name"
            },
            {
                "type":"grok",
                "field":"full_name",
                "pattern": "%{WORD:grok_first} %{WORD:grok_second}"
            },
            {
                "type": "append",
                "field": "full_name",
                "append": " hello   "
            },
            {
                "type": "rename",
                "field": "full_name",
                "new_name": "full_name_hello"
            },
            {
                "type": "join",
                "field": "phones_uk",
                "separator": ","
            },
            {
                "type": "lowercase",
                "field": "full_name_hello"
            },
            {
                "type": "set",
                "field": "my_field",
                "value": "my_value"
            },
            {
                "type": "trim_space",
                "field": "full_name_hello"
            },
            {
                "type": "split",
                "field": "full_name_hello",
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
        ]
    }"#;

    let expected = Vec::from(
        r#"{"age":43,"full_name_hello":["john","doe","hello"],"grok_first":"John","grok_second":"Doe","my_field":"_value","name":"John","phones":["+44 1234567","+44 2345678"],"phones_uk":"+44 1234567,+44 2345678","surname":"Doe","to_sort":[8,4,3],"to_sort_s":["asdasd","qweqw","were"]}"#,
    );

    let channel = Channel::new_vec(Vec::from(mutators_json_array)).unwrap();

    let res = parse_and_modify(data.as_bytes(), &channel);
    let a = std::str::from_utf8(res.unwrap().as_ref())
        .unwrap()
        .to_string();
    let b = std::str::from_utf8(expected.as_ref()).unwrap().to_string();
    assert_eq!(a, b)
}

#[test]
fn test_parser_input_plain_text() {
    use crate::channels::channel::parse_and_modify;
    #[cfg(test)]
    use crate::channels::channel::Channel;

    let data = r#"hello world"#;

    let mutators_json_array = r#"{
        "name": "my_channel",
        "channel": [{
                "type":"grok",
                "field":"_plain_input",
                "pattern": "%{WORD:grok.first} %{WORD:grok.second}"
            },
            {
                "type": "join",
                "field": ["grok.first", "grok.second"],
                "separator": ", ",
                "new_field": "full_msg"
            }
        ]
    }"#;

    let expected =
        Vec::from(r#"{"full_msg":"hello, world","grok.first":"hello","grok.second":"world"}"#);

    let channel = Channel::new_vec(Vec::from(mutators_json_array)).unwrap();

    let res = parse_and_modify(data.as_bytes(), &channel);
    let a = std::str::from_utf8(res.unwrap().as_ref())
        .unwrap()
        .to_string();
    let b = std::str::from_utf8(expected.as_ref()).unwrap().to_string();
    assert_eq!(a, b)
}

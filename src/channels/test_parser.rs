#[cfg(test)]
use crate::channels::parser::{Processors, parse_and_modify};

#[test]
fn test_parser() {
    let data = r#"{"name":"John Doe","age":43,"delete":"this","phones":["+44 1234567","+44 2345678"],"phones_uk":["+44 1234567","+44 2345678"],"to_sort":[4,3,8],"to_sort_s":["were","asdasd","qweqw"]                  }"#;

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

    let expected = r#"{"age":43,"my_field":"_value","name_hello":["john","doe","hello"],"phones":["+44 1234567","+44 2345678"],"phones_uk":"+44 1234567,+44 2345678","to_sort":[8,4,3],"to_sort_s":["asdasd","qweqw","were"]}"#;

    let mods = Processors::new(mo.to_string()).unwrap();

    for _ in 0..10 {
        let res = parse_and_modify(data, &mods);
        assert_eq!(expected, res.unwrap())
    }
}
mod storage;
mod conversions;

use std::env;
use sledge::components::api;
use sledge::components::storage::{get_storage, Storage};

#[test]
fn test_get_by_id() {
    let mut st = match env::var("STORAGE") {
        Ok(selected_storage) => get_storage(selected_storage.as_str(), format!("/tmp/{}", selected_storage).as_ref()),
        Err(e) => panic!("Couldn't read STORAGE ({})", e),
    };

    do_insertions(&mut st);

    let key = "1".to_string();
    let app = api::new(st);
    let retrieval_result = app.get_by_id(key.clone());
    assert_eq!(retrieval_result.unwrap(), Some("hello".to_string()));
}

pub fn test_items() -> Vec<(String, String)> {
    vec![("1".to_string(), "hello".to_string()),
         ("3".to_string(), "ula".to_string()),
         ("7".to_string(), "yoda".to_string()),
         ("2".to_string(), "world".to_string()),
         ("5".to_string(), "tesla".to_string()),
         ("4".to_string(), "tyrion".to_string()),
         ("6".to_string(), "rocco".to_string())]
}

pub fn do_insertions(st: &mut Box<dyn Storage>) {
    for test_item in test_items() {
        st.put(test_item.0, test_item.1).unwrap()
    }
}


#[test]
fn assert_from_beginning() {
    let mut st = match env::var("STORAGE") {
        Ok(selected_storage) => get_storage(selected_storage.as_str(), format!("/tmp/{}", selected_storage).as_ref()),
        Err(e) => panic!("Couldn't read STORAGE ({})", e),
    };

    do_insertions(&mut st);

    let output = vec![
        ("1".to_string(), "hello".to_string()),
        ("2".to_string(), "world".to_string()),
        ("3".to_string(), "ula".to_string()),
        ("4".to_string(), "tyrion".to_string()),
        ("5".to_string(), "tesla".to_string()),
        ("6".to_string(), "rocco".to_string()),
        ("7".to_string(), "yoda".to_string()),
    ];

    let mut res = st.since("01".to_string()).unwrap();
    for i in 0..7 {
        let cur = res.next();
        match cur {
            Some(kv) => assert_eq!(kv.key, output.get(i).unwrap().0),
            None => break
        }
    }
    for kv in res {
        println!("Key: {}, Value: {}", kv.key, kv.value)
    }
}

use bytes::Bytes;
use sledge::components::storage::Storage;
use sledge::components::kv::KV;

mod storage;


pub fn test_items_sorted() -> Vec<(String, Bytes)> {
    let mut is = test_items();
    is.sort();
    is
}

pub fn test_items() -> Vec<(String, Bytes)> {
    vec![("1".to_string(), bytes::Bytes::from("hello")),
         ("3".to_string(), bytes::Bytes::from("ula")),
         ("7".to_string(), bytes::Bytes::from("yoda")),
         ("2".to_string(), bytes::Bytes::from("world")),
         ("5".to_string(), bytes::Bytes::from("tesla")),
         ("4".to_string(), bytes::Bytes::from("tyrion")),
         ("6".to_string(), bytes::Bytes::from("rocco"))]
}

pub fn do_insertions(keyspace: Option<String>, st: &mut Box<dyn Storage + Send + Sync>) {
    for (x, y) in test_items().into_iter() {
        st.put(keyspace.clone(), x, y).unwrap()
    }
}

pub fn check_iterators_equality(x: impl Iterator<Item=KV>, y: impl Iterator<Item=(String, Bytes)>) {
    let zip = x.zip(y);
    let mut total = 0;
    for (x, y) in zip {
        assert_eq!(x, y.0);
        total += 1;
    }

    assert_ne!(total, 0)
}
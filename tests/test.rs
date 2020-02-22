use sledge::components::storage::Storage;
use sledge::components::kv::KV;

mod storage;


pub fn test_items_sorted() -> Vec<(String, String)> {
    let mut is = test_items();
    is.sort();
    is
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

pub fn do_insertions(st: &mut Box<dyn Storage + Send + Sync>) {
    for test_item in test_items() {
        st.put(test_item.0, test_item.1).unwrap()
    }
}

pub fn check_iterators_equality(x: impl Iterator<Item=KV>, y: impl Iterator<Item=(String,String)>){
    let zip = x.zip(y);

    for (x, y) in zip {
        assert_eq!(x, y.0)
    }
}
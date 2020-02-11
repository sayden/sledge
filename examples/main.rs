use sledge::storage::sled::Sled;
use sledge::storage::void::Void;
use std::env;
use sledge::storage::rocks::Rocks;
use sledge::components::{api};
use sledge::components::storage::{Storage, Options, Bound};

fn main() {
    let st = match env::var("STORAGE") {
        Ok(selected_storage) => get_storage(selected_storage.as_str(), format!("/tmp/{}", selected_storage).as_ref()),
        Err(e) => panic!("Couldn't read STORAGE ({})", e),
    };

    st.put("01".to_string(), "hello".to_string()).unwrap();
    st.put("02".to_string(), "world".to_string()).unwrap();
    st.put("03".to_string(), "ula".to_string()).unwrap();
    st.put("04".to_string(), "tyrion".to_string()).unwrap();
    let results = st.since("01".to_string());
    match results {
        Ok(values) => {
            for kv in values {
                println!("Key: {}, Value: {}", kv.key, kv.value)
            }
        }
        Err(e) => println!("Error chungo: {}", e)
    }
    println!("-------------------------");

//    let insertion_result = st.put("01", "world");
//    print_put_result(insertion_result);

    let key = "01".to_string();
    let a = st.since_until("mario".to_string(),"ula".to_string(),Some(vec![Options::Bounds(Bound::Infinite)]));
    for i in a.unwrap(){
        println!("Since 02 to 04! {}", i);
    }

    println!("-------------------------");
    let app = api::new(st);
    let retrieval_result = app.get_by_id(key.clone());
    match retrieval_result {
        Ok(o) => match o {
            Some(v) => println!("get_by_id: {}", v),
            None => println!("key '{}' not found", key)
        },
        Err(e) => println!("{}", e),
    }
}

fn get_storage(s: &str, p: &str) -> Box<dyn Storage> {
    match s {
        "sled" => Sled::new(p.to_string()),
        "rocksdb" => Rocks::new(p.to_string()),
        "void" => Void::new(),
        _ => panic!("storage '{}' not found", s),
    }
}

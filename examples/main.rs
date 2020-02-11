use sledge::storage::sled::Sled;
use sledge::storage::void::Void;
use std::env;
use sledge::storage::rocks::Rocks;
use sledge::components::{api};
use sledge::components::storage::Storage;

fn main() {
    let st = match env::var("STORAGE") {
        Ok(selected_storage) => get_storage(selected_storage.as_str(), format!("/tmp/{}", selected_storage).as_ref()),
        Err(e) => panic!("Couldn't read STORAGE ({})", e),
    };

    st.put("01", "hello").unwrap();
    st.put("02", "world").unwrap();
    st.put("03", "ula").unwrap();
    st.put("04", "tyrion").unwrap();
    let results = st.since("01");
    match results {
        Ok(values) => {
            for kv in values {
                println!("Key: {}, Value: {}", kv.key, kv.value)
            }
        }
        Err(e) => println!("Error chungo: {}", e)
    }

//    let insertion_result = st.put("01", "world");
//    print_put_result(insertion_result);

    let app = api::new(st);

    let key = "01";
    let retrieval_result = app.get_by_id(key);
    match retrieval_result {
        Ok(o) => match o {
            Some(v) => println!("{}", v),
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

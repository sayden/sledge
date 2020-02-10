mod sleddb;
mod rocks;
mod storage;
mod framework;
mod app;
mod transformations;
mod errors;

#[macro_use]
extern crate failure;

use crate::rocks::Rocks;
use crate::sleddb::Sled;
use crate::storage::Storage;
use std::env;

fn main() {
    let st = match env::var("STORAGE") {
        Ok(selected_storage) => get_storage(selected_storage.as_str(), format!("/tmp/{}", selected_storage).as_ref()),
        Err(e) => panic!("Couldn't read STORAGE ({})", e),
    };

    let framework = framework::new(st);

    framework.put("01", "hello").unwrap();
    framework.put("02", "world").unwrap();
    framework.put("03", "ula").unwrap();
    framework.put("04", "tyrion").unwrap();
    let results = framework.range("01");
    match results {
        Ok(values) => {
            for value in values {
                println!("Key: {}, Value: {}", value.0, value.1)
            }
        }
        Err(e) => println!("Error chungo: {}", e)
    }

//    let insertion_result = framework.put("01", "world");
//    print_put_result(insertion_result);

    let app = app::new(framework);

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
        _ => panic!("storage '{}' not found", s),
    }
}
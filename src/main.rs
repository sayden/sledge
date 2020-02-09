mod sleddb;
mod rocks;
mod storage;
mod framework;
mod app;

mod app_errors;

use crate::rocks::Rocks;
use std::error::Error;
use crate::sleddb::Sled;
use crate::storage::Storage;

fn print_result_option<E: std::marker::Sized + Error>(res: Result<Option<String>, E>) {
    match res {
        Ok(o) => match o {
            Some(msg) => println!("Ok: {}", msg),
            None => println!("not found"),
        },
        Err(e) => println!("error: {}", e.to_string()),
    }
}

fn main() {
    let st = get_storage(StorageTypes::Sled, "/tmp/sled");
//    let st = get_storage(StorageTypes::Rocks, "/tmp/rocks");

    let framework = framework::new(st);
    let app = app::new(framework);


//    let insertion_result = app.put("hello", "world");
//    print_result_option(insertion_result);

    let retrieval_result = app.get_by_id("hello");
    print_result_option(retrieval_result);
}

#[derive(Debug)]
enum StorageTypes {
    Sled,
    Rocks
}

fn get_storage(s: StorageTypes, p: &str) -> Box<dyn Storage> {
    match s {
        StorageTypes::Sled => Sled::new(p.to_string()),
        StorageTypes::Rocks => Rocks::new(p.to_string()),
    }
}
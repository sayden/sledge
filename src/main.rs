mod sleddb;
mod rocks;
mod storage;
mod framework;

use sleddb::Sled;
use crate::framework::{FrameworkError, Framework};

use crate::storage::{Storage, DbError};
use crate::rocks::Rocks;
use std::error::Error;

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
//    let sdb: &Storage = &Sled::new(String::from("/tmp/sled"));
    let sdb: &dyn Storage = &Rocks::new(String::from("/tmp/rocks"));
    let framework: &dyn Framework = &framework::FrameworkV1 { storage: sdb };


    let insertion_result = framework.put("hello", "world");
    print_result_option(insertion_result);

    let retrieval_result = framework.get("hello");
    print_result_option(retrieval_result);
}

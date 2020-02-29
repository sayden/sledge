#[cfg(test)]
mod db {
    use bytes::Bytes;
use crate::{do_insertions, test_items, test_items_sorted, check_iterators_equality};
    use sledge::components::storage::{get_storage};

    #[test]
    fn test_put() {
        let path = "/tmp/test_put";
        let mut st = get_storage("sled", path);

        do_insertions(None, &mut st);

        let a = st.since_until(None, "2".to_string(), "4".to_string()).unwrap();

        let mut tested_items: Vec<(String, Bytes)> = test_items();
        tested_items.sort();

        let zip = tested_items.iter().skip(1).take(2).zip(a);

        for (x, y) in zip {
            assert_eq!(y, x.0)
        }

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_put_in_tree() {
        let path = "/tmp/test_put_in_tree";
        let mut st = get_storage("sled", path);

        let tree_name = "my_tree".to_string();
        let tree_name_2 = "other_tree".to_string();
        do_insertions(Some(tree_name.clone()), &mut st);

        st.start(Some(tree_name_2)).unwrap();

        let a = st.start(Some(tree_name.clone())).unwrap();

        let items_sorted: Vec<(String, Bytes)> = test_items_sorted();

        check_iterators_equality(a, items_sorted.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_data() {
        env_logger::init();

        log::info!("hello");
        let path = "/tmp/storage";
        let mut st = get_storage("sled", path);

        let tree_name = "my_db".to_string();

        st.get(Some(tree_name), "12345".to_string()).unwrap();

        let items_sorted: Vec<(String, Bytes)> = test_items_sorted();

        log::info!("iterating");
        for (a,b) in items_sorted {
            log::info!("test_data: {} {}", a, std::str::from_utf8(b.bytes()).unwrap())
        }

        std::fs::remove_dir_all(path).unwrap();
    }
}
#[cfg(test)]
mod rocks {
    use sledge::components::storage::get_storage;

    use crate::{do_insertions, check_iterators_equality, test_items_sorted};

    #[test]
    fn test_since_until() {
        let path = "/tmp/test_since_until";
        let mut st = get_storage("rocksdb", path);

        do_insertions(&mut st);

        let a = st.since_until("2".to_string(), "4".to_string()).unwrap();

        let tested_items: Vec<(String, String)> = test_items_sorted();

        check_iterators_equality(a,tested_items.into_iter().skip(1).take(2));

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_start() {
        let path = "/tmp/test_start";
        let mut st = get_storage("rocksdb", path);

        do_insertions(&mut st);

        let a = st.start().unwrap();

        let tested_items: Vec<(String, String)> = test_items_sorted();

        check_iterators_equality(a,tested_items.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_end() {
        let path = "/tmp/test_end";
        let mut st = get_storage("rocksdb", path);

        do_insertions(&mut st);

        let a = st.end().unwrap();

        let mut tested_items: Vec<(String, String)> = test_items_sorted();
        tested_items.reverse();

        check_iterators_equality(a,tested_items.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }
}
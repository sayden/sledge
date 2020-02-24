#[cfg(test)]
mod rocks {
    use sledge::components::storage::{get_storage, Error};

    use crate::{do_insertions, check_iterators_equality, test_items_sorted};
    use sledge::storage::rocks::{Rocks};

    #[test]
    fn test_since_until() {
        let path = "/tmp/test_since_until";
        let mut st = get_storage("rocksdb", path);

        do_insertions(None, &mut st);

        let a = st.since_until(None, "2".to_string(), "4".to_string()).unwrap();

        let tested_items: Vec<(String, String)> = test_items_sorted();

        check_iterators_equality(a, tested_items.into_iter().skip(1).take(2));

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_start() {
        let path = "/tmp/test_start";
        let mut st = get_storage("rocksdb", path);

        do_insertions(None, &mut st);

        let a = st.start(None).unwrap();

        let tested_items: Vec<(String, String)> = test_items_sorted();

        check_iterators_equality(a, tested_items.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_end() {
        let path = "/tmp/test_end";
        let mut st = get_storage("rocksdb", path);

        do_insertions(None, &mut st);

        let a = st.end(None).unwrap();

        let mut tested_items: Vec<(String, String)> = test_items_sorted();
        tested_items.reverse();

        check_iterators_equality(a, tested_items.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_with_cf() {
        env_logger::init();

        let path = "/tmp/test_with_cf";
        let mut st = get_storage("rocksdb", path);
        let cf1 = "cf1".to_string();
        let cf2 = "cf2".to_string();

        st.create_keyspace(cf1.clone()).unwrap();
        do_insertions(Some(cf1.clone()), &mut st);

        match st.start(Some(cf2)){
            Ok(_) => assert!(false),
            Err(e) => match e {
                Error::Preparing(s) => assert_eq!(s, "keyspace with name cf2 not found"),
                _ => assert!(false, "expected error not found")
            }
        }

        let a = st.start(Some(cf1.clone())).unwrap();

        let items_sorted: Vec<(String, String)> = test_items_sorted();

        check_iterators_equality(a, items_sorted.into_iter());

        std::fs::remove_dir_all(path).unwrap();
    }
}
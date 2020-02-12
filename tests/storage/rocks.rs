#[cfg(test)]
mod db {
    use sledge::components::storage::get_storage;

    use crate::{do_insertions, test_items};

    #[test]
    fn test_rocks() {
        let path = "/tmp/test_put";
        let mut st = get_storage("rocksdb", path);

        do_insertions(&mut st);

        let a = st.since_until("2".to_string(), "4".to_string()).unwrap();

        let mut tested_items: Vec<(String, String)> = test_items();
        tested_items.sort();

        let zip = tested_items.iter().skip(1).take(2).zip(a);

        for (x, y) in zip {
            assert_eq!(y, x.0)
        }
        std::fs::remove_dir_all(path).unwrap();
    }
}
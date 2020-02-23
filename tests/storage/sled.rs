#[cfg(test)]
mod db {
    use crate::{do_insertions, test_items};
    use sledge::components::storage::{get_storage};
    use sledge::conversions::vector::convert_vec_pairs_u8;

    #[test]
    fn test_put() {
        let path = "/tmp/test_put";
        let mut st = get_storage("sled", path);
        do_insertions(None, &mut st);

        let a = st.since_until(None, "2".to_string(), "4".to_string()).unwrap();

        let mut tested_items: Vec<(String, String)> = test_items();
        tested_items.sort();

        let zip= tested_items.iter().skip(1).take(2).zip(a);

        for (x, y) in zip {
            assert_eq!(y, x.0)
        }

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_put_in_tree(){
        let path = "/tmp/test_put_in_tree";
        let db = sled::open(path).unwrap();
        let tree_a  = db.open_tree("tree_a").unwrap();
        let tree_b  = db.open_tree("tree_b").unwrap();

        tree_a.insert("99", "hello").unwrap();
        tree_b.insert("00", "world").unwrap();

        for i in db.range("00"..){
            let b = i.unwrap();
            let c =convert_vec_pairs_u8(b.0.as_ref(), b.1.as_ref()).unwrap();
            println!("{} {}", c.key,c.value);
        }
        println!("done");

        for i in tree_a.range("00"..){
            let b = i.unwrap();
            let c =convert_vec_pairs_u8(b.0.as_ref(), b.1.as_ref()).unwrap();
            println!("{} {}", c.key,c.value);
        }
        println!("done");

        for i in tree_b.range("00"..){
            let b = i.unwrap();
            let c =convert_vec_pairs_u8(b.0.as_ref(), b.1.as_ref()).unwrap();
            println!("{} {}", c.key,c.value);
        }
        println!("done");

        std::fs::remove_dir_all(path).unwrap();

    }
}
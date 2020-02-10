#[macro_export]
macro_rules! print_err_and_none {
    ($e:expr) => {{
        {
            println!("{}", $e);
            None
        }
    }}
}

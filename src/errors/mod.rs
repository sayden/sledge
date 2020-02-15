#[macro_export]
macro_rules! print_err_and_none {
    ($e:expr) => {{
        {
            error!("{}", $e);
            None
        }
    }}
}

pub mod storage;
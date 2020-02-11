use anyhow::Error;

pub(crate) type SledgeIterator = dyn Iterator<Item=KV>;

#[derive(Debug)]
pub struct KV {
    pub key: String,
    pub value: String,
}

impl PartialEq for KV {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

pub enum Bound {
    Limit(u32),
    Key(String),
    KeyEqualsValue(KV),
    Infinite,
}

pub enum Options {
    Bounds(Bound)
}


/**
* Types of range operations:
* (window) since upper bounded with a limit based on count
* (window) since upper bounded with a limit on a key found (or limit)
* since: infinite, no stop if possible
* since: infinite until signal
* backwards from key down to a limit based on count
* backwards from key down to a key found (or limit)
* backwards from key, infinite
* backwards from key, infinite until signal
*
* Examples:
* since("my_key", Bound::Infinite)
* since("my_key", Bound::Limit(100))
* since("my_key", Bound::Key("stop_in_this_key"))
* since("my_key", Bound::Key("stop_in_this_key"), Bound::Limit(10))
* since("my_key", Bound::Key("stop_in_this_key"), Bound::KV(KV{key:"stop_if_this_key",value:"has_this_value"))
*
* backwards("my_key", Bound::Infinite)
* backwards("my_key", Bound::Limit(100))
* backwards("my_key", Bound::Key("stop_in_this_key"))
* backwards("my_key", Bound::Key("stop_in_this_key"), Bound::Limit(10))
* backwards("my_key", Bound::Key("stop_in_this_key"), Bound::KV(KV{key:"stop_if_this_key",value:"has_this_value"))
*/
pub trait Storage {
    fn get(&self, s: &str) -> Result<Option<String>, Error>;
    fn put(&self, k: &str, v: &str) -> Result<(), Error>;

    fn since(&self, k: &str) -> Result<Box<SledgeIterator>, Error>;
    fn since_until(&self, k: &str, k2: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error>;

    fn reverse(&self, k: &str) -> Result<Box<SledgeIterator>, Error>;
    fn reverse_until(&self, k: &str, opt: Box<[Options]>) -> Result<Box<SledgeIterator>, Error>;
}
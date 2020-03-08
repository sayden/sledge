use rocksdb::DBRawIterator;

use crate::channels::parser::{Channel, parse_and_modify_u8};
use crate::components::simple_pair::SimplePair;

pub type SledgeIterator = Box<dyn Iterator<Item=SimplePair> + Send + Sync>;

pub fn with_channel(iter: SledgeIterator, ch: Option<Channel>, omit_errors:Option<bool>) -> SledgeIterator {
    if ch.is_none() {
        return iter
    }

    let ch = ch.unwrap();

    let thread_iter: SledgeIterator = box iter
        .flat_map(move |sp| {
            parse_and_modify_u8(sp.v.as_slice(), &ch, omit_errors)
                .map(|x| SimplePair::new_vec(sp.k, x))
        });
    thread_iter
}


pub struct RawIteratorWrapper<'a> {
    pub inner: DBRawIterator<'a>,
}

impl Iterator for RawIteratorWrapper<'_> {
    type Item = SimplePair;

    fn next<'b>(&mut self) -> Option<<Self as Iterator>::Item> {
        self.inner.next();
        if !self.inner.valid() {
            return None;
        }

        let k = self.inner.key()?;
        let v = self.inner.value()?;

        Some(SimplePair::new_u8(k, v))
    }
}
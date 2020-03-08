use bytes::Bytes;
use rocksdb::DBRawIterator;

use crate::channels::channel::{Channel, parse_and_modify_u8};
use crate::components::simple_pair::SimplePair;
use crate::server::query::Query;

pub type SledgeIterator = Box<dyn Iterator<Item=SimplePair> + Send + Sync>;

pub fn with_channel(iter: SledgeIterator, ch: Option<Channel>, q: &Option<Query>) -> SledgeIterator {
    if ch.is_none() {
        return iter;
    }

    let omit_errors = q.as_ref().and_then(|ref q| q.omit_errors).unwrap_or_else(|| false);

    let ch = ch.unwrap();

    let thread_iter: SledgeIterator = box iter
        .flat_map(move |sp| {
            parse_and_modify_u8(sp.v.as_slice(), &ch, omit_errors)
                .map(|x| SimplePair::new_vec(sp.k, x))
        });
    thread_iter
}

pub fn with_channel_for_single_value(val: Bytes, ch: Option<Channel>, q: &Option<Query>) -> Bytes {
    if ch.is_none() {
        return val;
    }

    let omit_errors = q.as_ref().and_then(|ref q| q.omit_errors).unwrap_or_else(|| false);
    let ch = ch.unwrap();

    let res = parse_and_modify_u8(val.as_ref(), &ch, omit_errors);

    res.map(Bytes::from).unwrap_or(val)
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
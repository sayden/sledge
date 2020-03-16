use rocksdb::DBRawIterator;

use crate::components::simple_pair::SimplePair;

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

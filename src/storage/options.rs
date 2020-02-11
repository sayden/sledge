use crate::components::storage::KV;

pub trait UntilExt: Iterator {
    fn until(self, kv: KV) -> Until<Self>
        where
            Self: Sized,
            Self: Iterator<Item=KV> {
        Until::new(self, kv)
    }
}

#[derive(Debug)]
pub struct Until<I: Iterator<Item=KV>> {
    iter: I,
    flag: bool,
    kv: KV,
    stop: bool,
}

impl<I: Iterator<Item=KV>> Until<I> {
    pub(super) fn new(iter: I, kv: KV) -> Until<I> {
        Until { iter, flag: false, kv, stop: false }
    }
}


// LimitTo(u32)
// Skip(u32)
// Since(String)
// SinceKV(KV)
// Until(String)
// UntilKV(KV)
// Infinite
// ExcludeFirst
// ExcludeLast
impl<I: Iterator<Item=KV>> Iterator for Until<I> {
    type Item = KV;

    #[inline]
    fn next(&mut self) -> Option<KV> {
        let x = self.iter.next()?;
        if self.stop {
            return None;
        }

        self.stop = self.kv.key != x.key;

        Some(x)
    }
}

impl<I: Iterator> UntilExt for I {}
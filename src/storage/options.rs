use crate::components::kv::KV;

pub trait ProcessOrStop<X> where
    X: PartialEq + Eq {
    fn process_or_stop(&mut self, cur: X) -> Option<X>;
}


pub struct After<T> {
    found: bool,
    pub compared: T,
}

impl<T> After<T> {
    pub fn new(compared: T) -> Self {
        After { found: false, compared }
    }
}

impl<T, X> ProcessOrStop<X> for After<T>
    where X: PartialEq<T> + Eq {
    fn process_or_stop(&mut self, cur: X) -> Option<X> {
        if self.found {
            return Some(cur);
        } else {
            self.found = cur == self.compared;
            if self.found {
                return Some(cur);
            }
        }

        None
    }
}


pub struct Until<T> {
    stop: bool,
    pub compared: T,
}

impl<T> Until<T> {
    pub fn new(compared: T) -> Self {
        Until { stop: false, compared }
    }
}

impl<T, X> ProcessOrStop<X> for Until<T>
    where X: PartialEq<T> + Eq {
    fn process_or_stop(&mut self, cur: X) -> Option<X> {
        if self.stop { return None; }
        self.stop = cur == self.compared;
        return Some(cur);
    }
}



pub struct Limit<T> {
    stop: bool,
    pub compared: T,

    limit: u32,
    limit_total: u32,
}

impl<T> Limit<T> {
    pub fn new(compared: T, limit: u32) -> Self {
        Limit { stop: false, limit, limit_total: 0, compared }
    }
}

impl<T, X> ProcessOrStop<X> for Limit<T>
    where X: PartialEq<T> + Eq {
    fn process_or_stop(&mut self, cur: X) -> Option<X> {
        if self.stop { return None; }
        self.limit_total += 1;
        self.stop = self.limit_total >= self.limit;
        Some(cur)
    }
}



pub trait UntilExt: Iterator {
    fn until(self, processor: Box<dyn ProcessOrStop<KV>>) -> UntilIter<Self>
        where
            Self: Sized,
            Self: Iterator<Item=KV> {
        UntilIter::new(self, processor)
    }
}

pub struct UntilIter<I: Iterator<Item=KV>> {
    iter: I,
    processor: Box<dyn ProcessOrStop<KV>>,

}

impl<I: Iterator<Item=KV>> UntilIter<I> {
    pub(super) fn new(iter: I, processor: Box<dyn ProcessOrStop<KV>>) -> UntilIter<I> {
        UntilIter { iter, processor} }
}

// LimitTo(u32)
// Skip(u32)
// Since(String)
// SinceKV(KV)

// Until(String)
// UntilKV(KV)
// Infinite

impl<I: Iterator<Item=KV>> Iterator for UntilIter<I> {
    type Item = KV;

    #[inline]
    fn next(&mut self) -> Option<KV> {
        let cur = self.iter.next()?;
        self.processor.process_or_stop(cur)
    }
}

impl<I: Iterator> UntilExt for I {}
use crate::components::storage::{Storage, Options, KV, SledgeIterator};
use anyhow::Error;
use std::ops::Try;

pub struct Void {}

impl Void {
    pub fn new() -> Box<dyn Storage> {
        Box::new(Void {})
    }
}

trait TakeWhile2Ext: Iterator {
    fn take_while2<P>(self, predicate: P) -> TakeWhile2<Self, P>
        where
            Self: Sized,
            P: FnMut(&Self::Item) -> bool,
    {
        TakeWhile2::new(self, predicate)
    }
}

pub struct TakeWhile2<I, P> {
    iter: I,
    flag: bool,
    predicate: P,
    stop: bool,
}

impl<I, P> TakeWhile2<I, P> {
    pub(super) fn new(iter: I, predicate: P) -> TakeWhile2<I, P> {
        TakeWhile2 { iter, flag: false, predicate, stop: false }
    }
}

//impl<I: fmt::Debug, P> fmt::Debug for TakeWhile2<I, P> {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        f.debug_struct("TakeWhile2").field("iter", &self.iter).field("flag", &self.flag).finish()
//    }
//}

impl<I: Iterator, P> Iterator for TakeWhile2<I, P>
    where
        P: FnMut(&I::Item) -> bool, {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.stop {
            self.flag = true;
            return None
        }

        if self.flag {
            None
        } else {
            let x = self.iter.next()?;
            if (self.predicate)(&x) {
                Some(x)
            } else {
                self.stop = true;
                Some(x)
            }
        }
    }
}

impl<I: Iterator> TakeWhile2Ext for I {}

trait TakeWhile3Ext: Iterator {
    fn take_while3<P>(self, predicate: P) -> TakeWhile3<Self, P>
        where
            Self: Sized,
            P: FnMut(&Self::Item) -> bool,
    {
        TakeWhile3::new(self, predicate)
    }
}

pub struct TakeWhile3<I, P> {
    iter: I,
    flag: bool,
    predicate: P,
    stop: bool,
}

impl<I, P> TakeWhile3<I, P> {
    pub(super) fn new(iter: I, predicate: P) -> TakeWhile3<I, P> {
        TakeWhile3 { iter, flag: false, predicate, stop: false }
    }
}

//impl<I: fmt::Debug, P> fmt::Debug for TakeWhile3<I, P> {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        f.debug_struct("TakeWhile3").field("iter", &self.iter).field("flag", &self.flag).finish()
//    }
//}

impl<I: Iterator, P> Iterator for TakeWhile3<I, P>
    where
        P: FnMut(&I::Item) -> bool, {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        if self.stop {
            self.flag = true;
            return None
        }

        if self.flag {
            None
        } else {
            let x = self.iter.next()?;
            if (self.predicate)(&x) {
                Some(x)
            } else {
                self.stop = true;
                Some(x)
            }
        }
    }
}

impl<I: Iterator> TakeWhile3Ext for I {}

impl Storage for Void {
    fn get(&self, _: String) -> Result<Option<String>, Error> {
        Ok(Some("void get".to_string()))
    }

    fn put(&self, _: String, _: String) -> Result<(), Error> {
        Ok(())
    }

    fn since(&self, _: String) -> Result<Box<SledgeIterator>, Error> {
        let v = vec![1, 2, 3, 4, 5].into_iter();
        Ok(Box::new(v.map(|x| (KV { key: format!("{}", x), value: format!("{}", x) }))))
    }

    fn since_until(&self, k: String, k2: String, opt: Option<Vec<Options>>) -> Result<Box<SledgeIterator>, Error> {
        let v = vec![KV { key: "hello".to_string(), value: "hello_value".to_string() }, KV { key: "world".to_string(), value: "world_value".to_string() }, KV { key: "mario".to_string(), value: "mario_value".to_string() }, KV { key: "ula".to_string(), value: "ula_value".to_string() }, KV { key: "tyrion".to_string(), value: "tyrion_value".to_string() }, KV { key: "tesla".to_string(), value: "tesla_value".to_string() }];
        let res = v.into_iter()
            .take_while3(move |x| *x.key != k2);

//        let res = v.iter().until(KV { key: "mario".to_string(), value: "".to_string() });

        Ok(Box::new(res))
    }

    fn reverse(&self, k: String) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }

    fn reverse_until(&self, k: String, opt: Option<Vec<Options>>) -> Result<Box<SledgeIterator>, Error> {
        unimplemented!()
    }
}

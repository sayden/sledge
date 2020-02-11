use crate::components::storage::KV;

pub trait UntilExt: Iterator {
    fn until(self, until_kv: KV, limit: u32, skip: u32, until_string: String, after_key: String, after_kv: KV) -> Until<Self>
        where
            Self: Sized,
            Self: Iterator<Item=KV> {
        Until::new(self, until_kv, limit, skip, until_string, after_key, after_kv)
    }
}

#[derive(Debug)]
pub struct Until<I: Iterator<Item=KV>> {
    iter: I,

    stop: bool,

    after_key: String,
    after_kv: KV,
    found: bool,

    until_kv: KV,
    until_key: String,

    limit: u32,
    limit_total: u32,
    stop_by_limit:bool,

    skip: u32,
    skipped_total: u32,

    exclude_first: bool,
    exclude_last: bool,
}

impl<I: Iterator<Item=KV>> Until<I> {
    pub(super) fn new(iter: I, until_kv: KV, limit: u32, skip: u32, until_string: String, after_key: String, after_kv: KV) -> Until<I> {
        Until {
            iter,
            until_kv,
            stop: false,
            limit,
            limit_total: 0,
            stop_by_limit: false,
            skip,
            skipped_total: 0,
            exclude_first: false,
            until_key: until_string,
            found: false,
            after_key,
            after_kv,
            exclude_last: false,
        }
    }
}

// LimitTo(u32)
// Skip(u32)
// Since(String)
// SinceKV(KV)

// Until(String)
// UntilKV(KV)
// Infinite

impl<I: Iterator<Item=KV>> Iterator for Until<I> {
    type Item = KV;

    #[inline]
    fn next(&mut self) -> Option<KV> {
        let mut cur = self.iter.next()?;
        if self.stop || self.stop_by_limit{
            return None;
        }

        if self.limit != 0 {
            cur = self.limit(cur)?
        }
        if self.skip != 0 {
            cur = self.skip_first(cur)?
        }
        if !self.after_key.is_empty() {
            cur = self.after_key(cur)?
        }
        if !self.after_kv.key.is_empty() {
            cur = self.after_kv(cur)?
        }
        if !self.after_kv.key.is_empty() {
            cur = self.after_key(cur)?
        }
        if !self.until_kv.key.is_empty() {
            cur = self.until_kv(cur)?
        }
        if !self.until_key.is_empty() {
            cur = self.until_key(cur)?
        }

        Some(cur)
//        self.stop = self.until_kv.key != cur.key;
//        Some(x)
    }
}

impl<I: Iterator<Item=KV>> Until<I> {
    pub fn after_kv(&mut self, cur: KV) -> Option<KV> {
        self.found = self.after_kv.key == cur.key && self.after_kv.value == cur.value;

        if self.found {
            return Some(cur);
        }

        None
    }
    pub fn after_key(&mut self, cur: KV) -> Option<KV> {
        if self.found {
            return Some(cur);
        } else {
            self.found = self.after_key == cur.key;
            if self.found {
                return Some(cur)
            }
        }

        None
    }

    pub fn skip_first(&mut self, cur: KV) -> Option<KV> {
        self.skipped_total += 1;

        if self.skipped_total >= self.skip {
            return Some(cur);
        }

        None
    }

    pub fn limit(&mut self, cur: KV) -> Option<KV> {
        self.limit_total += 1;
        self.stop_by_limit = self.limit_total >= self.limit;
        Some(cur)
    }

    pub fn until_key(&mut self, current: KV) -> Option<KV> {
        self.stop = self.until_key == current.key;
        return Some(current);
    }

    pub fn until_kv(&mut self, current: KV) -> Option<KV> {
        self.stop = self.until_kv.key == current.key && self.until_kv.value == current.value;
        return Some(current);
    }
}

impl<I: Iterator> UntilExt for I {}
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KV {
    pub key: String,
    pub value: String,
}

impl KV {
    pub fn empty() -> Self {
        KV { key: "".to_string(), value: "".to_string() }
    }
}

impl PartialEq<String> for KV {
    fn eq(&self, other: &String) -> bool {
        self.key == *other
    }
}

impl PartialOrd for KV {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.key == other.key {
            return Some(Ordering::Equal);
        } else if self.key > other.key {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Less)
    }
}

impl Ord for KV {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.key == other.key {
            return Ordering::Equal;
        } else if self.key > other.key {
            return Ordering::Greater;
        }

        Ordering::Less
    }
}

impl fmt::Display for KV {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Key: {}, Value: {}", self.key, self.value)
    }
}
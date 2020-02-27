use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct KV {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl KV {
    pub fn empty() -> Self {
        KV { key: Vec::new(), value: Vec::new() }
    }
}

impl PartialEq<String> for KV {
    fn eq(&self, other: &String) -> bool {
        self.key == other.as_bytes()
    }
}

impl PartialEq<(String, String)> for KV {
    fn eq(&self, x: &(String, String)) -> bool {
        self.key == x.0.as_bytes() && self.value == x.1.as_bytes()
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
        let x = std::str::from_utf8(self.key.as_slice())
            .unwrap_or_default();
        let y = std::str::from_utf8(self.value.as_slice())
            .unwrap_or_default();
        write!(f, "Key: {}, Value: {}", x, y)
    }
}
pub mod mutators;

pub mod remove;
pub mod upper_lower_case;
pub mod join;
pub mod append;
pub mod rename;
pub mod set;
pub mod split;
pub mod grok;
pub mod trim_spaces;
pub mod trim;
pub mod sort;

pub mod channel;
mod test_parser;
pub(crate) mod error;
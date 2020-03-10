#![feature(box_syntax)]
#![feature(in_band_lifetimes)]
#![feature(bool_to_option)]

extern crate log;

#[macro_use]
extern crate anyhow;

pub mod components;
pub mod channels;
pub mod server;
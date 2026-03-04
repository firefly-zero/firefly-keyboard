#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;
extern crate firefly_rust;

mod luxboard_lite;

pub use luxboard_lite::*;

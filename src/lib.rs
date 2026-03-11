#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;
extern crate firefly_rust;

mod keyboard;

pub use keyboard::*;

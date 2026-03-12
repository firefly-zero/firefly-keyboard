#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::wildcard_imports
)]

extern crate alloc;
extern crate firefly_rust;

mod keyboard;
mod layout;

pub use keyboard::*;
pub use layout::*;

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod builder;
mod macros;
mod shared_impl;
mod slice;
mod slice_mut;

pub use builder::*;
pub use shared_impl::{SliceBase, SliceError};
pub use slice::*;
pub use slice_mut::*;

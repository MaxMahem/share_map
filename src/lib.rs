#![doc = include_str!("../README.md")]
mod handle;
mod iter;
mod share_map;

pub use handle::Handle;
pub use iter::Iter;
pub use share_map::{DuplicateKeyError, ShareMap};

pub use frozen_collections::{Len, MapIteration, MapQuery};

#[cfg(test)]
mod tests;

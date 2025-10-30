#![doc = include_str!("../README.md")]

mod handle;
mod share_map;

pub use handle::Handle;
pub use share_map::{DuplicateKeyError, Iter, ShareMap};

#[cfg(feature = "serde")]
pub use share_map::ensure_unqiue;

pub use frozen_collections::{Len, MapIteration, MapQuery};

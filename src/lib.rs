#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::match_bool)]
#![allow(clippy::multiple_crate_versions)]

mod handle;
mod share_map;

pub use handle::Handle;
pub use share_map::{DuplicateKeyError, Iter, ShareMap};

#[cfg(feature = "serde")]
pub use share_map::ensure_unqiue;

pub use frozen_collections::{Len, MapIteration, MapQuery};

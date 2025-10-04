#![doc = include_str!("../README.md")]
mod frozen_map;
mod hold;
mod swap_map;
mod value_ref;

pub use frozen_map::{DuplicateKeyError, FrozenMap, IntoIter};
pub use hold::Hold;
pub use swap_map::SwapMap;
pub use value_ref::ValueRef;

pub use frozen_collections::{Len, MapIteration, MapQuery};

#[cfg(test)]
pub(crate) type UnitResultAny = Result<(), Box<dyn std::error::Error>>;

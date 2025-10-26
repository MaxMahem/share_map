#![doc = include_str!("../README.md")]
mod frozen_map;
mod swap_map;
mod value;
mod value_ref;

pub use frozen_map::FrozenMap;
pub use swap_map::{DuplicateKeyError, SwapMap};
pub use value::Value;
pub use value_ref::ValueRef;

pub use frozen_collections::{Len, MapIteration, MapQuery};

#[cfg(test)]
pub(crate) type UnitResultAny = Result<(), Box<dyn std::error::Error>>;

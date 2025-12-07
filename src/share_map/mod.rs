mod iter;
#[cfg(feature = "serde")]
mod serde;
#[allow(clippy::module_inception)]
mod share_map;

pub use iter::Iter;
#[cfg(feature = "serde")]
pub use serde::ensure_unqiue;
pub use share_map::{DuplicateKeyError, ShareMap};

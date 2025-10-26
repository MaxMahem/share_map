#[allow(clippy::module_inception)]
mod frozen_map;
//mod into_iter;
mod iter;

pub use frozen_map::{DuplicateKeyError, FrozenMap};
//pub use into_iter::IntoIter;
pub use iter::Iter;

#[cfg(doc)]
use crate::SwapMap;
use std::ops::Deref;
use std::sync::Arc;

/// A reference to a value in a [SwapMap].
#[derive(Debug)]
pub struct ValueRef<V> {
    store: Arc<[V]>,
    index: usize,
}

impl<V> ValueRef<V> {
    pub(crate) fn new(store: Arc<[V]>, index: usize) -> Self {
        debug_assert!(index < store.len());
        Self { store, index }
    }
}

impl<V> Deref for ValueRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        // SAFETY: `index` is guaranteed to be in bounds
        unsafe { self.store.get_unchecked(self.index) }
    }
}

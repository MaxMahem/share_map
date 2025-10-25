#[cfg(doc)]
use crate::SwapMap;
use std::ops::Deref;
use std::sync::Arc;

/// A reference to a value in a [SwapMap].
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

impl<V: std::fmt::Debug> std::fmt::Debug for ValueRef<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<V> Deref for ValueRef<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        // Panic safety: `index` is guaranteed to be in bounds
        &self.store[self.index]
    }
}

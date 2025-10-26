use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use arc_swap::ArcSwap;
use fluent_result::IntoResult;
use frozen_collections::{Len, MapIteration, MapQuery};
use tap::Pipe;

use crate::frozen_map::{DuplicateKeyError, FrozenMap};
use crate::{Value, ValueRef};

/// A thread-safe, lock-free frozen map that is immutable, but allows atomic swapping of the
/// entire map contents.
///
/// [SwapMap] provides a way to maintain a shared, immutable map that can be atomically
/// replaced with a new version. Readers can access the current version without blocking
/// writers, and writers can atomically replace the entire map without affecting ongoing reads.
///
/// This is particularly useful for scenarios with frequent reads and occasional bulk updates,
/// such as configuration reloading, caching, or periodically rebuilt lookup tables.
///
/// # Thread safety
///
/// [SwapMap] is thread-safe and can be used concurrently from multiple threads. The underlying
/// swapping mechanism is provided by [ArcSwap]. All performance implications and limitations
/// of [ArcSwap] apply.
///
/// # Map Type
///
/// [SwapMap] can be configured to use a custom map type for lookup. By default it uses [HashMap],
/// but can use any type that implements [MapQuery], [Len], and [FromIterator].
///
/// [SwapMap] depends upon the map implementation for most hash-map operations, including the
/// constrains on the key type `K` (typically [Hash](std::hash::Hash) + [Eq]), and what alternate
/// types can be used to query keys in [SwapMap::get] and [SwapMap::contains_key] (for example,
/// [HashMap] allow query for any type that implements [Borrow](std::borrow::Borrow) for the key
/// type).
///
/// Note: the provided `Map` type must be from key (`K`) to internal value index (a `usize`)
/// (i.e. `HashMap<K, usize>`), not key to value.
///
/// # Retrieved [ValueRef]s
///
/// [SwapMap::get] produces [ValueRef]s that provide immutable reference access to values stored in
/// the map. A [ValueRef] is guranteed to remain valid for its lifetime and will always point into
/// the map state it was created from — it will not invalidate when the map is swapped or reflect
/// changes in the map due to the swap.
///
/// No mutable access is provided to stored values. If values use interior mutability, callers
/// must ensure those mutations are thread-safe. Such changes will be visible to all [ValueRef]s
/// using the same snapshot of the map, but not new [ValueRef]s created after [SwapMap::store].
///
/// # Iteration
///
/// [SwapMap] does not provide any iteration over the map. To iterate, call [SwapMap::snapshot]
/// and use the iterators provided by [FrozenMap].
///
/// # Ownership
///
/// [SwapMap] is thread safe and provides shared ownership of its data. Callers can invoke
/// [SwapMap::snapshot] at any time to obtain an [Arc] wrapped [FrozenMap], with the underlying
/// data.
///
/// Because ownership is shared in this way, acquiring exclusive ownership of a [FrozenMap] is
/// nontrivial. Since [SwapMap] itself owns a reference, any operation that seeks exclusive
/// ownership must inherently own and consume the [SwapMap].
///
/// With that in mind, to acquire exclusive access, consider one of the following [SwapMap]
/// consuming methods:
///
/// 1. [SwapMap::into_snapshot] — Returns the [FrozenMap] if no other snapshots exist; otherwise
///    returns [None].
/// 2. [SwapMap::try_into_snapshot] — Returns a [Value]: [Value::Owned] if exclusive, or
///    [Value::Shared] with a wrapping [Arc].
/// 3. [SwapMap::into_snapshot_or_clone] — Returns the [FrozenMap] if exclusive, or a clone if
///    shared. Only available if `K`, `V`, and `Map` implement [Clone].
///
/// In addition, the [Arc] wrapped [FrozenMap] returned by [SwapMap::swap] is not guranteed to have
/// sole ownership, as previous snapshots may exists sharing ownership. However [ValueRef]s created
/// by [SwapMap::get] do *not* cause shared ownership in this way.
///
/// # Type Parameters
///
/// - `K`: The key type stored in the map
/// - `V`: The value type stored in the map.
/// - `Map`: The map used to map keys to internal indices.
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use swap_map::{SwapMap, ValueRef};
///
/// // Create a new empty SwapMap
/// let swap_map = SwapMap::<&str, i32>::new();
/// swap_map.store([("key1", 42), ("key2", 100)])?;
///
/// // Read data
/// let value: ValueRef<i32> = swap_map.get("key1").ok_or("Key not found")?;
/// assert_eq!(*value, 42);
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct SwapMap<K, V, Map = HashMap<K, usize>> {
    datastore: ArcSwap<FrozenMap<K, V, Map>>,
}

impl<K, V, Map> SwapMap<K, V, Map> {
    /// Creates a new empty [SwapMap].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::new();
    /// assert!(swap_map.is_empty());
    /// ```
    pub fn new() -> Self
    where
        Map: Default,
    {
        Self {
            datastore: ArcSwap::default(),
        }
    }

    /// Creates a new [SwapMap] from the provided key-value pairs.
    ///
    /// # Type Parameters
    ///
    /// - `I`: An iterator over the key-value pairs to be stored.
    /// - `KIn`: A type that can be converted to `K`.
    ///
    /// # Errors
    ///
    /// Fails with [DuplicateKeyError] if the provided data contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{DuplicateKeyError, SwapMap};
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// assert_eq!(swap_map.len(), 2);
    ///
    /// // duplicate key's error
    /// let swap_map_err = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key1", 100)]);
    /// assert_eq!(swap_map_err.unwrap_err(), DuplicateKeyError);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_pairs<I>(iter: I) -> Result<Self, DuplicateKeyError>
    where
        Map: FromIterator<(K, usize)> + Len,
        I: IntoIterator<Item = (K, V)>,
    {
        FrozenMap::from_pairs(iter).map(|snapshot_map| Self {
            datastore: ArcSwap::from_pointee(snapshot_map),
        })
    }

    /// Creates a new [SwapMap] from the provided map.
    ///
    /// # Type Parameters
    ///
    /// - `MapIn`: A map that implements [MapIteration], and can be converted to `K`.
    /// - `KIn`: A type that can be converted to `K`.
    ///
    /// # Panics
    ///
    /// Panics if the provided map contains duplicate keys. This should not be possible, but the
    /// Map contract cannot gurantee this.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use swap_map::SwapMap;
    ///
    /// let hash_map = HashMap::from([("key1", 42), ("key2", 100)]);
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_map(hash_map);
    ///
    /// assert_eq!(swap_map.len(), 2);
    /// ```
    pub fn from_map<MapIn>(map: MapIn) -> Self
    where
        Map: FromIterator<(K, usize)> + Len,
        MapIn: MapIteration<K, V>,
    {
        Self::from_pairs(map).expect("Map should not contain duplicate keys")
    }

    /// Atomically replaces the entire map contents with the provided key-value pairs.
    ///
    /// This operation atomicly replaces the data in the map with the new data provided.
    /// All subsequent reads will see the new data, while any existing [ValueRef]s will continue
    /// to see the old data until they complete.
    ///
    /// # Type Parameters
    ///
    /// - `I`: An iterator over the key-value pairs to be stored.
    /// - `KIn`: A type that can be converted to `K`.
    ///
    /// # Errors
    ///
    /// Fails with [DuplicateKeyError] if the provided data contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{SwapMap, DuplicateKeyError, ValueRef};
    ///
    /// let swap_map = SwapMap::<&str, i32>::new();
    /// swap_map.store([("key1", 42), ("key2", 100)])?;
    ///
    /// // stored data available to new reads
    /// let value1: ValueRef<i32> = swap_map.get("key1").ok_or("key not found")?;
    /// assert_eq!(*value1, 42);
    ///
    /// swap_map.store([("key1", 21), ("key2", 200)])?;
    ///
    /// // old ValueRef's still valid, point to old data.
    /// assert_eq!(*value1, 42);
    ///
    /// // new data available to new reads.
    /// let value2: ValueRef<i32> = swap_map.get("key1").ok_or("key not found")?;
    /// assert_eq!(*value2, 21);
    ///
    /// // duplicate key's error
    /// let err = swap_map.store([("key1", 42), ("key1", 100)]).unwrap_err();
    /// assert_eq!(err, DuplicateKeyError);
    /// # Ok(())
    /// # }
    /// ```
    pub fn store<I>(&self, iter: I) -> Result<(), DuplicateKeyError>
    where
        Map: FromIterator<(K, usize)> + Len,
        I: IntoIterator<Item = (K, V)>,
    {
        let new = FrozenMap::from_pairs(iter).map(Arc::new)?;
        self.datastore.store(new);
        Ok(())
    }

    /// Atomically replaces the entire map contents with the provided key-value pairs,
    /// and returns the old data as a [FrozenMap].
    ///
    /// This operation atomicly replaces the data in the map with the new data provided.
    /// All subsequent reads will see the new data, while any existing [ValueRef]s will continue
    /// to see the old data until they complete.
    ///
    /// # Type Parameters
    ///
    /// - `I`: An iterator over the key-value pairs to be stored.
    /// - `KIn`: A type that can be converted to `K`.
    ///
    /// # Returns
    ///
    /// Returns the old data as a [FrozenMap].
    ///
    /// # Errors
    ///
    /// Fails with [DuplicateKeyError] if the provided data contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{DuplicateKeyError, SwapMap, ValueRef};
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let old_data = swap_map.swap([("key1", 21), ("key2", 200)])?;
    ///
    /// // swap_map points to new data
    /// let value1: ValueRef<i32> = swap_map.get("key1").ok_or("Key not found")?;
    /// assert_eq!(*value1, 21);
    ///
    /// // old_data points to old data
    /// let value2: Option<&i32> = old_data.get("key1");
    /// assert_eq!(value2, Some(&42));
    ///
    /// let err = swap_map.swap([("key1", 42), ("key1", 100)]).unwrap_err();
    /// assert_eq!(err, DuplicateKeyError);
    /// # Ok(())
    /// # }
    /// ```
    pub fn swap<I>(&self, iter: I) -> Result<Arc<FrozenMap<K, V, Map>>, DuplicateKeyError>
    where
        Map: FromIterator<(K, usize)> + Len,
        I: IntoIterator<Item = (K, V)>,
    {
        let new = FrozenMap::from_pairs(iter).map(Arc::new)?;
        self.datastore.swap(new).into_ok()
    }

    /// Retrieves a snapshot of the current map data.
    ///
    /// This snapshot will remain valid as long as it lives, even if the producing [SwapMap] is
    /// dropped or its data is replaced, however it will not reflect any changes made to the map
    /// afterwards.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?;
    ///
    /// let snapshot = swap_map.snapshot();
    /// assert_eq!(snapshot.get("key1"), Some(&42));
    ///
    /// // snapshot is still valid and points to the same data after a store
    /// swap_map.store([("key1", 21), ("key2", 200)])?;
    /// assert_eq!(snapshot.get("key1"), Some(&42));
    ///
    /// // snapshot is still valid and points to the same data after a drop
    /// drop(swap_map);
    /// assert_eq!(snapshot.get("key1"), Some(&42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn snapshot(&self) -> Arc<FrozenMap<K, V, Map>> {
        self.datastore.load().clone()
    }

    /// Converts the [SwapMap] into a [FrozenMap] if there are no other outstanding snapshots.
    ///
    /// Returns [None] if there are other snapshots.
    ///
    /// Note this consumes the [SwapMap] regardless of whether there are other snapshots.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let other_snapshot = swap_map.snapshot();
    ///
    /// // other_snapshot is still valid, so into_snapshot returns None
    /// let none = swap_map.into_snapshot();
    /// assert_eq!(none.is_none(), true);
    ///
    /// // After rebinding, swap_map now has no other snapshots
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    ///
    /// // swap_map has no other snapshots, so into_snapshot returns Some
    /// let snapshot = swap_map.into_snapshot();
    /// assert_eq!(snapshot.is_some(), true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_snapshot(self) -> Option<FrozenMap<K, V, Map>> {
        self.datastore.into_inner().pipe(Arc::into_inner)
    }

    /// Converts the [SwapMap] into a [FrozenMap].
    ///
    /// # Returns
    /// - [Value::Owned] if there are no other snapshots
    /// - [Value::Shared] if there are other snapshots
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let other_snapshot = swap_map.snapshot();
    ///
    /// // other_snapshot is still valid, so try_into_snapshot returns Shared
    /// let shared = swap_map.try_into_snapshot();
    /// assert!(shared.is_shared());
    ///
    /// // After rebinding, swap_map now has no other snapshots
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    ///
    /// // swap_map has no other snapshots, so try_into_snapshot returns Owned
    /// let owned = swap_map.try_into_snapshot();
    /// assert!(owned.is_owned());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_into_snapshot(self) -> Value<FrozenMap<K, V, Map>> {
        self.datastore.into_inner().into()
    }

    /// Converts the [SwapMap] into a [FrozenMap] if there are no other outstanding snapshots, clones
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let other_snapshot = swap_map.snapshot();
    ///
    /// // other_snapshot is still valid, so into_snapshot_or_clone returns a clone
    /// let clone = swap_map.into_snapshot_or_clone();
    /// assert_eq!(clone.get("key1"), Some(&42));
    ///
    /// // After rebinding, swap_map now has no other snapshots
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    ///
    /// // swap_map has no other snapshots, so into_snapshot_or_clone returns original
    /// let original = swap_map.into_snapshot_or_clone();
    /// assert_eq!(original.get("key1"), Some(&42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_snapshot_or_clone(self) -> FrozenMap<K, V, Map>
    where
        K: Clone,
        V: Clone,
        Map: Clone,
    {
        self.datastore.into_inner().pipe(Arc::unwrap_or_clone)
    }

    /// Retrieves a reference to the value associated with the given key.
    ///
    /// Returns [`Some(ValueRef<V>)`](Some) if the key exists, or [None] otherwise.
    ///
    /// The returned [ValueRef] provides thread-safe access to the value without additional guards
    /// or locks. It will remain valid as long as it is in scope, even if the underlying map is
    /// dropped or replaced, however it will not reflect any changes made after a [SwapMap::store]
    /// or [SwapMap::swap] call.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{SwapMap, ValueRef};
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let value: ValueRef<i32> = swap_map.get("key1").ok_or("Key not found")?;
    /// assert_eq!(*value, 42);
    ///
    /// // value is still valid after a store, swap, or drop
    /// drop(swap_map);
    /// assert_eq!(*value, 42);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<ValueRef<V>>
    where
        Map: MapQuery<Q, usize>,
    {
        self.datastore.load().get_value_ref(key)
    }

    /// Checks if the map contains a specific key.
    ///
    /// Returns `true` if the key exists in the current map version, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// assert_eq!(swap_map.contains_key("key1"), true);
    /// assert_eq!(swap_map.contains_key("key3"), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Map: MapQuery<Q, usize>,
    {
        self.datastore.load().contains_key(key)
    }

    /// Returns the number of key-value pairs in the current map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// assert_eq!(swap_map.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.datastore.load().len()
    }

    /// Checks if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let swap_map = SwapMap::<&str, i32>::new();
    /// assert!(swap_map.is_empty());
    ///
    /// swap_map.store([("key1", 42), ("key2", 100)])?;
    /// assert!(!swap_map.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.datastore.load().is_empty()
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug, Map: MapIteration<K, usize>> std::fmt::Debug
    for SwapMap<K, V, Map>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.snapshot().iter()).finish()
    }
}

impl<K, V, Map> From<HashMap<K, V>> for SwapMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    fn from(map: HashMap<K, V>) -> Self {
        // SAFETY: HashMap should ensure that there are no duplicates
        unsafe { Self::from_pairs(map).unwrap_unchecked() }
    }
}

impl<K, V, Map> From<BTreeMap<K, V>> for SwapMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    fn from(map: BTreeMap<K, V>) -> Self {
        // SAFETY: HashMap should ensure that there are no duplicates
        unsafe { Self::from_pairs(map).unwrap_unchecked() }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use crate::SwapMap;
    use crate::UnitResultAny;

    #[test]
    fn test_swap_map_iteration_pattern() {
        let btree_map = BTreeMap::from([("key1", 42), ("key2", 100)]);
        let swap_map: SwapMap<&str, i32, BTreeMap<&str, usize>> = btree_map.clone().into();
        let snapshot = swap_map.snapshot();

        let swap_vec: Vec<_> = snapshot.iter().collect();
        let btree_vec: Vec<_> = btree_map.iter().collect();

        assert_eq!(swap_vec, btree_vec);
    }

    /// Test against BTreeMap for reliability because HashMap does not guarantee iteration order
    #[test]
    fn test_swap_map_debug_matches_btreemap() {
        let btree_map = BTreeMap::from([("key", 42), ("key2", 100)]);
        let swap_map: SwapMap<&str, i32, BTreeMap<&str, usize>> = btree_map.clone().into();

        let swap_debug = format!("{:?}", swap_map);
        let btree_debug = format!("{:?}", btree_map);

        assert_eq!(swap_debug, btree_debug);
    }

    #[test]
    fn test_swap_map_get_value_ref_does_not_share_ownership() -> UnitResultAny {
        let swap_map = SwapMap::<&str, i32>::from_pairs([("key", 42)])?;

        let _value_ref = swap_map.get("key").ok_or("Key not found")?;

        let some_snapshot = swap_map.try_into_snapshot();

        assert!(some_snapshot.is_owned());

        Ok(())
    }

    #[test]
    fn test_swap_map_swap_returned_value_does_not_share_ownership() -> UnitResultAny {
        let swap_map = SwapMap::<&str, i32>::from_pairs([("key", 42)])?;

        let old_snapshot = swap_map.swap([("key", 21)])?;

        let unwrapped_snapshot = Arc::try_unwrap(old_snapshot);

        assert!(unwrapped_snapshot.is_ok());

        Ok(())
    }

    #[test]
    fn test_swap_map_snapshot_shares_ownership() -> UnitResultAny {
        let swap_map = SwapMap::<&str, i32>::from_pairs([("key", 42)])?;

        let snapshot = swap_map.snapshot();

        let unwrapped_snapshot = Arc::try_unwrap(snapshot);

        assert!(unwrapped_snapshot.is_err());

        Ok(())
    }
}

use std::sync::Arc;
use std::{collections::HashMap, ops::Index};

use fluent_result::IntoResult;
use frozen_collections::{Len, MapIteration, MapQuery};

#[cfg(doc)]
use crate::SwapMap;
use crate::ValueRef;
use crate::frozen_map::BorrowIter;

/// An immutable snapshot of a map's contents that supports efficient, shared read access.
///
/// This type is intentionally immutable: once a [FrozenMap] is created it never changes. That
/// makes it safe to share across threads and to hand out lightweight handles into the snapshot
/// (see [FrozenMap::get_value_ref]).
///
/// # Map Dependent Behavior
///
/// The `Map` implementation defines many elements of behavior, including the constraints on the
/// key type (`K`). What types can be used to query keys in [FrozenMap::get],
/// [FrozenMap::contains_key], and [FrozenMap::get_value_ref].
///
/// # Map Iteration
///
/// Behavior during iteration for any value iteration that includes the key ([FrozenMap::keys],
/// [FrozenMap::iter], [FrozenMap::into_iter]) is dependent on the map used for the lookup.
/// Enumeration of values ('V') alone ([FrozenMap::values]) is always in order provided during
/// construction.
///
/// Any owned enumeration including values ([FrozenMap::into_iter]) requires that the values
/// (`V`) be [Clone] and requires a cloneing of the values.
///
/// # Type Parameters
/// - `K`: The key type stored in the map
/// - `V`: The value type stored in the map.
/// - `Map`: The map used to map keys to internal indices.
///
/// # Examples Note
///
/// Because [FrozenMap] is not user constructable, all examples use [SwapMap::snapshot] for
/// construction, which returns a `Arc<FrozenMap>`.
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use swap_map::SwapMap;
///
/// let swap_map = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
/// let snapshot = swap_map.snapshot();
/// assert_eq!(snapshot.get("key1"), Some(&42));
/// assert_eq!(snapshot.get("key2"), Some(&100));
/// # Ok(())
/// # }
/// ```
#[derive(derive_more::Debug, Clone)]
pub struct FrozenMap<K, V, Map = HashMap<K, usize>> {
    index_map: Map,
    store: Arc<[V]>,
    #[debug(skip)]
    _marker: std::marker::PhantomData<K>,
}

impl<K, V, Map: Default> Default for FrozenMap<K, V, Map> {
    fn default() -> Self {
        Self {
            index_map: Map::default(),
            store: Arc::new([]),
            _marker: std::marker::PhantomData,
        }
    }
}

/// An error indicating that a duplicate key was found in the provided data.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Duplicate key found")]
pub struct DuplicateKeyError;

impl<K, V, Map> FrozenMap<K, V, Map> {
    /// Creates a new [FrozenMap] from the provided key-value pairs.
    ///
    /// # Type Parameters
    ///
    /// - `I`: An iterator over the key-value pairs to be stored.
    ///
    /// # Errors
    ///
    /// Fails with [DuplicateKeyError] if the provided data contains duplicate keys.
    pub(crate) fn from_pairs<I>(iter: I) -> Result<Self, DuplicateKeyError>
    where
        Map: FromIterator<(K, usize)> + Len,
        I: IntoIterator<Item = (K, V)>,
    {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();

        let mut store = Vec::with_capacity(lower);
        let mut temp = Vec::with_capacity(lower);

        for (key, value) in iter {
            let index = store.len();
            store.push(value);
            temp.push((key.into(), index));
        }

        let index_map = Map::from_iter(temp);

        match index_map.len() == store.len() {
            false => Err(DuplicateKeyError),
            true => Self {
                index_map,
                store: store.into_boxed_slice().into(),
                _marker: std::marker::PhantomData,
            }
            .into_ok(),
        }
    }

    /// Returns the value associated with the given key, if it exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?.snapshot();
    /// assert_eq!(snapshot.get("key1"), Some(&42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Map: MapQuery<Q, usize>,
    {
        self.index_map.get(key).map(|index| &self.store[*index])
    }

    /// Returns the value associated with the given key as a [ValueRef], if it exists.
    ///
    /// The returned [ValueRef] will remain valid for as long as they live, even if the producing
    /// [FrozenMap] is dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?.snapshot();
    ///
    /// let value_ref = snapshot.get_value_ref("key1").ok_or("Key not found")?;
    ///
    /// assert_eq!(*value_ref, 42);
    ///
    /// // value_ref is still valid after drop
    /// drop(snapshot);
    ///
    /// assert_eq!(*value_ref, 42);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_value_ref<Q: ?Sized>(&self, key: &Q) -> Option<ValueRef<V>>
    where
        Map: MapQuery<Q, usize>,
    {
        self.index_map
            .get(key)
            .map(|index| ValueRef::new(self.store.clone(), *index))
    }

    /// Checks if the map contains a specific key.
    ///
    /// Key equality is determined by the `Map` implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?.snapshot();
    ///
    /// assert_eq!(snapshot.contains_key("key1"), true);
    /// assert_eq!(snapshot.contains_key("key3"), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Map: MapQuery<Q, usize>,
    {
        self.index_map.contains_key(key)
    }

    /// Returns an iterator over the key-value pairs in the map.
    ///
    /// Order of iteration is dependent on the `Map` implementation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use assert_unordered::*;
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<i32, i32>::from_pairs([(15, 42), (32, 100)])?.snapshot();
    ///
    /// let mut pairs: Vec<(&i32, &i32)> = snapshot.iter().collect();
    ///
    /// assert_eq_unordered!(pairs, vec![(&15, &42), (&32, &100)]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> BorrowIter<'_, K, V, Map::Iterator<'_>>
    where
        Map: MapIteration<K, usize>,
    {
        BorrowIter::new(self.index_map.iter(), &self.store)
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Order of iteration is dependent on the `Map` implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use assert_unordered::*;
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?.snapshot();
    ///
    /// let keys: Vec<&&str> = snapshot.keys().collect();
    ///
    /// assert_eq_unordered!(keys, vec![&"key1", &"key2"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn keys(&self) -> Map::KeyIterator<'_>
    where
        Map: MapIteration<K, usize>,
    {
        self.index_map.keys()
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Unlike [HashMap::values], this method is `O(n:len)`, not `O(n:capacity)`.
    ///
    /// Values are returned in the same order they were given.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    /// use assertables::*;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?.snapshot();
    ///
    /// let values: Vec<&i32> = snapshot.values().collect();
    ///
    /// assert_iter_eq!(values, vec![&42, &100]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn values(&self) -> std::slice::Iter<'_, V> {
        self.store.iter()
    }

    /// Consumes the [FrozenMap] and returns a key (`K`) iterator.
    ///
    /// Order of iteration is dependent on the `Map` implementation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use assert_unordered::*;
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?
    ///     .into_snapshot()
    ///     .ok_or("Multiple Owners")?;
    ///
    /// let keys: Vec<&str> = snapshot.into_keys().collect();
    ///
    /// assert_eq_unordered!(keys, vec!["key1", "key2"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_keys(self) -> Map::IntoKeyIterator
    where
        Map: MapIteration<K, usize>,
    {
        self.index_map.into_keys()
    }

    /// Consumes the [FrozenMap] and returns the value store.
    ///
    /// Value in the store are in the same order they were given.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::sync::Arc;
    /// use assertables::*;
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?
    ///      .into_snapshot().ok_or("Multiple Owners")?;
    ///
    /// let values: Arc<[i32]> = snapshot.into_values();
    ///
    /// assert_iter_eq!(values, vec![42, 100]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_values(self) -> Arc<[V]> {
        self.store
    }

    /// Returns the number of key-value pairs in the current map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?.snapshot();
    ///
    /// let len = snapshot.len();
    ///
    /// assert_eq!(len, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Checks if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::SwapMap;
    ///
    /// let snapshot = SwapMap::<&str, i32>::new().snapshot();
    /// assert_eq!(snapshot.is_empty(), true);
    ///
    /// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?.snapshot();
    /// assert_eq!(snapshot.is_empty(), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl<K, V: Clone, Map> IntoIterator for FrozenMap<K, V, Map>
where
    Map: MapIteration<K, usize>,
{
    type Item = (K, V);
    type IntoIter = crate::frozen_map::IntoIter<K, V, Map::IntoIter>;

    fn into_iter(self) -> Self::IntoIter {
        crate::frozen_map::IntoIter::new(self.index_map.into_iter(), self.store)
    }
}

impl<'a, K, V, Map> IntoIterator for &'a FrozenMap<K, V, Map>
where
    Map: MapIteration<K, usize>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = BorrowIter<'a, K, V, Map::Iterator<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        BorrowIter::new(self.index_map.iter(), &self.store)
    }
}

impl<K, V, Map> Index<K> for FrozenMap<K, V, Map>
where
    Map: Index<K, Output = usize>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.store[self.index_map[index]]
    }
}

#[cfg(test)]
mod tests {
    use assert_unordered::assert_eq_unordered;
    use assertables::*;

    use crate::FrozenMap;
    use crate::UnitResultAny;

    #[test]
    fn test_snapshot_map_from_pairs() -> UnitResultAny {
        let snapshot_map_ok = FrozenMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)]);

        assert_ok!(snapshot_map_ok);

        // duplicate key's error
        let snapshot_map_err = FrozenMap::<&str, i32>::from_pairs([("key1", 42), ("key1", 100)]);

        assert_err!(snapshot_map_err);
        Ok(())
    }

    #[test]
    fn test_map_snapshot_index() -> UnitResultAny {
        let snapshot = FrozenMap::<&str, i32>::from_pairs([("key1", 42)])?;

        assert_eq!(snapshot["key1"], 42);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_map_snapshot_invalid_index() {
        let snapshot = FrozenMap::<&str, i32>::from_pairs([("key1", 42)]).unwrap();

        assert_eq!(snapshot["key2"], 0);
    }

    #[test]
    fn test_map_snapshot_into_iter_owned() -> UnitResultAny {
        let snapshot = FrozenMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;

        let pairs: Vec<(&str, i32)> = snapshot.into_iter().collect();

        assert_eq_unordered!(pairs, vec![("key1", 42), ("key2", 100)]);

        Ok(())
    }

    #[test]
    fn test_map_snapshot_into_iter_borrowed() -> UnitResultAny {
        let snapshot = FrozenMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;

        let pairs: Vec<(&&str, &i32)> = (&snapshot).into_iter().collect();

        assert_eq_unordered!(pairs, vec![(&"key1", &42), (&"key2", &100)]);

        Ok(())
    }
}

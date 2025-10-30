use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::ops::Index;
use std::sync::Arc;

#[cfg(doc)]
use collect_failable::TryCollectEx;
use collect_failable::TryFromIterator;
use fluent_result::IntoResult;
use frozen_collections::{Len, MapIteration, MapQuery};

use crate::Handle;
use crate::Iter;

/// An immutable map's of values that supports shared read access and provides access to stable,
/// sharable value references ([`Handle`]s).
///
/// This type is intentionally immutable: once a [`ShareMap`] is created it never changes. It
/// is safe to share across threads and to hand out lightweight handles ([`Handle`]s) into the map
/// via [`ShareMap::get_handle`].
///
/// # Construction
///
/// Unless duplicate values are allowed, [`ShareMap::try_from_iter`] or the corresponding
/// [`TryCollectEx::try_collect_ex`] extension should be prefered for construction.
///
/// # Clone
///
/// Cloning involves a deep clone of keys, but a shallow copy of the values themselves.
///
/// # Map Iteration
///
/// Because ownership of values is shared, owned enumeration including values is not provided.
///
/// # Type Parameters
/// - `K`: The key type stored in the map
/// - `V`: The value type stored in the map.
/// - `Map`: The map used to map keys to internal indices.
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use share_map::ShareMap;
/// use collect_failable::TryCollectEx;
///
/// let map: ShareMap<_, _> = [("key1", 42), ("key2", 100)].into_iter().try_collect_ex()?;
/// assert_eq!(map.get("key1"), Some(&42));
/// assert_eq!(map.get("key2"), Some(&100));
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ShareMap<K, V, Map = HashMap<K, usize>> {
    index_map: Map,
    values: Arc<[V]>,
    _marker: std::marker::PhantomData<K>,
}

/// An error returned when duplicate keys are encountered during construction.
#[derive(Debug, thiserror::Error)]
#[error("duplicate key")]
pub struct DuplicateKeyError;

impl<K, V, Map> ShareMap<K, V, Map> {
    fn new(index_map: Map, values: Arc<[V]>) -> Self {
        Self {
            index_map,
            values,
            _marker: std::marker::PhantomData,
        }
    }

    /// Attempts to create a new [`ShareMap`] from the provided key-value pairs.
    ///
    /// # Errors
    ///
    /// Fails with [`DuplicateKeyError`] if the provided data contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{DuplicateKeyError, ShareMap};
    ///
    /// let map = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(map.get("key1"), Some(&42));
    /// assert_eq!(map.get("key2"), Some(&100));
    ///
    /// // duplicate key's error
    /// let err: DuplicateKeyError = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key1", 100)])
    ///     .expect_err("should be duplicate key");
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_iter<I>(iterable: I) -> Result<Self, DuplicateKeyError>
    where
        I: IntoIterator<Item = (K, V)>,
        Map: FromIterator<(K, usize)> + Len,
    {
        let (values, key_index_pairs): (Vec<_>, Vec<_>) = iterable
            .into_iter()
            .enumerate()
            .map(|(index, (key, value))| (value, (key, index)))
            .unzip();

        // convert the key_index_pairs into a map, this should remove duplicates
        let index_map = Map::from_iter(key_index_pairs);

        match index_map.len() == values.len() {
            true => Self::new(index_map, values.into()).into_ok(),
            false => Err(DuplicateKeyError),
        }
    }

    /// Returns the value associated with the given key, if it exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::ShareMap;
    ///
    /// let map = ShareMap::<&str, i32>::try_from_iter([("key1", 42)])?;
    /// let value: Option<&i32> = map.get("key1");
    ///
    /// assert_eq!(value, Some(&42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Map: MapQuery<Q, usize>,
    {
        self.index_map.get(key).map(|index| &self.values[*index])
    }

    /// Returns the value associated with the given key as a [`Handle`], if it exists.
    ///
    /// The returned [`Handle`] will never invalidate, even if the original [`ShareMap`] is
    /// dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{ShareMap, Handle};
    ///
    /// let map = ShareMap::<_, _>::try_from_iter([("key1", 42)])?;
    ///
    /// let handle: Handle<i32> = map.get_handle("key1").ok_or("Key not found")?;
    ///
    /// assert_eq!(*handle, 42);
    ///
    /// // handle is still valid after map is dropped
    /// drop(map);
    ///
    /// assert_eq!(*handle, 42);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_handle<Q: ?Sized>(&self, key: &Q) -> Option<Handle<V>>
    where
        Map: MapQuery<Q, usize>,
    {
        self.index_map
            .get(key)
            .map(|index| Handle::new(self.values.clone(), *index))
    }

    /// Checks if the map contains a specific key.
    ///
    /// Key equality is determined by the `Map` implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::ShareMap;
    ///
    /// let map = ShareMap::<_, _>::try_from_iter([("key1", 42)])?;
    ///
    /// assert_eq!(map.contains_key("key1"), true);
    /// assert_eq!(map.contains_key("key3"), false);
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
    /// use std::collections::BTreeMap;
    /// use share_map::ShareMap;
    ///
    /// // BTreeMap gurantees iteration order
    /// let data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(data)?;
    ///
    /// let map_keys: Vec<_> = map.iter().collect();
    /// let data_keys: Vec<_> = data.iter().map(|(k, v)| (k, v)).collect();
    ///
    /// assert_eq!(map_keys, data_keys);
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V, Map::Iterator<'_>>
    where
        Map: MapIteration<K, usize>,
    {
        Iter::new(self.index_map.iter(), &self.values)
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Order of iteration is dependent on the `Map` implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::collections::BTreeMap;
    /// use share_map::ShareMap;
    ///
    /// // BTreeMap gurantees iteration order
    /// let data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(data)?;
    ///
    /// let map_keys: Vec<_> = map.keys().collect();
    /// let data_keys: Vec<_> = data.iter().map(|(k, _)| k).collect();
    ///
    /// assert_eq!(map_keys, data_keys);
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
    /// Unlike [`HashMap::values`], this method is `O(n:len)`, not `O(n:capacity)`.
    ///
    /// Values iteration order is not defined.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::collections::HashSet;
    /// use share_map::ShareMap;
    ///
    /// let data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _>::try_from_iter(data)?;
    ///
    /// let map_values = map.values();
    ///
    /// // value order is not defined, so compare as sets
    /// let data_set: HashSet<_> = data.iter().map(|(_, v)| v).collect();
    /// let share_set: HashSet<_> = map_values.collect();
    /// assert_eq!(data_set, share_set);
    /// # Ok(())
    /// # }
    /// ```
    pub fn values(&self) -> std::slice::Iter<'_, V> {
        self.values.iter()
    }

    /// Consumes the [`ShareMap`] and returns a key (`K`) iterator.
    ///
    /// Order of iteration is dependent on the `Map` implementation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::collections::BTreeMap;
    /// use share_map::ShareMap;
    ///
    /// // BTreeMap gurantees iteration order
    /// let data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(data)?;
    ///
    /// let map_keys: Vec<_> = map.into_keys().collect();
    /// let data_keys: Vec<_> = data.into_iter().map(|(k, _)| k).collect();
    ///
    /// assert_eq!(map_keys, data_keys);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_keys(self) -> Map::IntoKeyIterator
    where
        Map: MapIteration<K, usize>,
    {
        self.index_map.into_keys()
    }

    /// Consumes the [`ShareMap`] and returns the value store.
    ///
    /// The order of the values is not defined.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::sync::Arc;
    /// use std::collections::HashSet;
    /// use share_map::ShareMap;
    ///
    /// let data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _>::try_from_iter(data)?;
    ///
    /// let map_values: Arc<[i32]> = map.into_values();
    ///
    /// // value order is not defined, so compare as sets
    /// let data_set: HashSet<_> = data.iter().map(|(_, v)| v).collect();
    /// let share_set: HashSet<_> = map_values.iter().collect();
    /// assert_eq!(data_set, share_set);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_values(self) -> Arc<[V]> {
        self.values
    }

    /// Returns the number of key-value pairs in the current map.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::ShareMap;
    ///
    /// let map = ShareMap::<&str, i32>::try_from_iter([("key1", 42), ("key2", 100)])?;
    ///
    /// let len = map.len();
    ///
    /// assert_eq!(len, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Checks if the map is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::ShareMap;
    ///
    /// let map = ShareMap::<(), ()>::default();
    /// assert_eq!(map.is_empty(), true);
    ///
    /// let map = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
    /// assert_eq!(map.is_empty(), false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<K: Debug, V: Debug, Map> Debug for ShareMap<K, V, Map>
where
    Map: MapIteration<K, usize>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

// manual implementation is necessary because #Derive thinks PhantomData requires K:Default
impl<K, V, Map: Default> Default for ShareMap<K, V, Map> {
    fn default() -> Self {
        Self {
            index_map: Map::default(),
            values: Arc::default(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<K, V, Map> Eq for ShareMap<K, V, Map>
where
    Map: MapQuery<K, usize> + MapIteration<K, usize>,
    V: Eq,
{
}

impl<K, V, Map> PartialEq for ShareMap<K, V, Map>
where
    Map: MapQuery<K, usize> + MapIteration<K, usize>,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // cannot perform cheap ptr arc equality check because PartialEq is not symmetric
        if self.values.len() != other.values.len() {
            return false;
        }

        self.iter().all(|(key, value)| {
            other
                .get(key)
                .is_some_and(|other_value| value == other_value)
        })
    }
}

impl<'a, K, V, Map> IntoIterator for &'a ShareMap<K, V, Map>
where
    Map: MapIteration<K, usize>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, Map::Iterator<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.index_map.iter(), &self.values)
    }
}

impl<K, Q, V, Map> Index<Q> for ShareMap<K, V, Map>
where
    Map: Index<Q, Output = usize>,
{
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        let index = self.index_map[index];
        &self.values[index]
    }
}

impl<K, V, Map, const N: usize> TryFrom<[(K, V); N]> for ShareMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    type Error = DuplicateKeyError;

    fn try_from(value: [(K, V); N]) -> Result<Self, Self::Error> {
        Self::try_from_iter(value)
    }
}

impl<K, V, Map> From<HashMap<K, V>> for ShareMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    fn from(value: HashMap<K, V>) -> Self {
        Self::try_from_iter(value).expect("HashMap should not contain duplicate keys")
    }
}

impl<K, V, Map> From<ShareMap<K, V, Map>> for HashMap<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
    Map: MapIteration<K, usize>,
{
    fn from(value: ShareMap<K, V, Map>) -> Self {
        value
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<K, V, Map> From<BTreeMap<K, V>> for ShareMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::try_from_iter(value).expect("Map should not contain duplicate keys")
    }
}

impl<K, V, Map> From<ShareMap<K, V, Map>> for BTreeMap<K, V>
where
    K: Eq + std::hash::Hash + Clone + Ord,
    V: Clone,
    Map: MapIteration<K, usize>,
{
    fn from(value: ShareMap<K, V, Map>) -> Self {
        value
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<K, V, Map> FromIterator<(K, V)> for ShareMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len + MapIteration<K, usize>,
{
    /// Creates a new [`ShareMap`] from an iterator of key-value pairs.
    ///
    /// Unless duplicate keys are allowed, prefer [`ShareMap::try_from_iter`] or the corresponding
    /// [`TryCollectEx::try_collect_ex`] extension instead.
    ///
    /// In the case of duplicate keys, the value stored depends on the map implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// use share_map::ShareMap;
    ///
    /// let map = ShareMap::<_, _>::from_iter([("key1", 1), ("key2", 2)]);
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(map["key1"], 1);
    /// assert_eq!(map["key2"], 2);
    ///
    /// // duplicate keys, value stored depends on the map implementation.
    /// // For HashMap, the last value seen is stored
    /// let map = ShareMap::<_, _, HashMap<_, _>>::from_iter([("key1", 1), ("key1", 2)]);
    /// assert_eq!(map.len(), 1);
    /// assert_eq!(map["key1"], 2);
    /// ```
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iterable: T) -> Self {
        let (mut values, mut key_index_pairs): (Vec<_>, Vec<_>) = iterable
            .into_iter()
            .enumerate()
            .map(|(index, (key, value))| (Some(value), (key, index)))
            .unzip();

        // converting the key_index_pairs into a map should remove duplicates
        let index_map: Map = key_index_pairs.drain(..).collect();

        match usize::cmp(&index_map.len(), &values.len()) {
            Ordering::Equal => {
                // PANIC SAFETY: all values in store are Some
                let store = values.into_iter().map(Option::unwrap).collect();
                Self::new(index_map, store)
            }
            Ordering::Greater => panic!("Invalid map implementation"),
            Ordering::Less => {
                // in the event of duplicates, rebuild the index_map and store
                let index_map_len = index_map.len();

                let (key_index_pairs, values) = index_map
                    .into_iter()
                    .enumerate()
                    .map(|(index, (key, old_index))| {
                        // PANIC SAFETY: all values in store are Some
                        ((key, index), values[old_index].take().unwrap())
                    })
                    // fold is used instead of zip to reuse key_index_pairs
                    .fold(
                        (key_index_pairs, Vec::with_capacity(index_map_len)),
                        |(mut key_index_pairs, mut new_values), (key_index_pair, value)| {
                            new_values.push(value);
                            key_index_pairs.push(key_index_pair);
                            (key_index_pairs, new_values)
                        },
                    );

                let index_map: Map = Map::from_iter(key_index_pairs);

                assert!(
                    index_map.len() == values.len() && values.len() == index_map_len,
                    "Invalid map implementation"
                );

                Self::new(index_map, values.into())
            }
        }
    }
}

impl<K, V, Map> TryFromIterator<(K, V)> for ShareMap<K, V, Map>
where
    Map: FromIterator<(K, usize)> + Len,
{
    type Error = DuplicateKeyError;

    /// Attempts to create a new [`ShareMap`] from the provided key-value pairs.
    ///
    /// # Errors
    ///
    /// Fails with [`DuplicateKeyError`] if the provided data contains duplicate keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{DuplicateKeyError, ShareMap};
    ///
    /// let test_data = [("key1", 42), ("key2", 100)];
    /// let map = ShareMap::<_, _>::try_from_iter(test_data)?;
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(map.get("key1"), Some(&42));
    /// assert_eq!(map.get("key2"), Some(&100));
    ///
    /// // duplicate key's error
    /// let test_data = [("key1", 42), ("key1", 100)];
    /// let err: DuplicateKeyError = ShareMap::<_, _>::try_from_iter(test_data)
    ///     .expect_err("should be duplicate key");
    /// # Ok(())
    /// # }
    /// ```
    fn try_from_iter<I>(iterable: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = (K, V)>,
    {
        ShareMap::try_from_iter(iterable)
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V, Map> serde::Deserialize<'de> for ShareMap<K, V, Map>
where
    K: Eq + std::hash::Hash + serde::Deserialize<'de>,
    V: serde::Deserialize<'de>,
    Map: FromIterator<(K, usize)> + Len,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::deserialize(deserializer).map(ShareMap::from)
    }
}

#[cfg(feature = "serde")]
impl<K, V, Map> serde::Serialize for ShareMap<K, V, Map>
where
    K: serde::Serialize,
    V: serde::Serialize,
    Map: MapIteration<K, usize>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self)
    }
}

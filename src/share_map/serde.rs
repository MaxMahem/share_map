use crate::{Len, MapIteration, ShareMap};

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
        std::collections::HashMap::deserialize(deserializer).map(ShareMap::from)
    }
}

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

/// Provides deserialization of a [`ShareMap`] that enforces that all keys are unique.
///
/// You can use this by annotating the type with `#[serde(with = "ensure_unqiue")]` or
/// by calling the [`ensure_unqiue::deserialize`] function directly.
///
/// # Example
///
/// ```rust
/// use share_map::{ShareMap, ensure_unqiue};
///
/// #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// struct TestContainer {
///     #[serde(with = "ensure_unqiue")]
///     map: ShareMap<String, u8>,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// // duplicate key will cause a data error
/// let serialized_data_with_duplicates = r#"{"map":{"key1":42,"key2":100,"key1":42}}"#;
/// let err = serde_json::from_str::<TestContainer>(serialized_data_with_duplicates).expect_err("should Err");
/// assert!(err.is_data());
///
/// // normal data can still be deserialized normally
/// let data = [("key1", 42), ("key2", 100)].map(|(k, v)| (k.to_string(), v));
/// let test_container = TestContainer { map: ShareMap::from_iter(data) };
///
/// let serialized = serde_json::to_string(&test_container)?;
/// let deserialized_container: TestContainer = serde_json::from_str(&serialized)?;
///
/// assert_eq!(test_container, deserialized_container);
/// # Ok(())
/// # }
/// ```
pub mod ensure_unqiue {
    use std::{hash::Hash, marker::PhantomData};

    use serde::Serialize;
    use tap::Pipe;

    use crate::{Len, ShareMap};

    /// Serializes the map. This method simply passes through to [`ShareMap::serialize`].
    ///
    /// # Errors
    ///
    /// Any errors from [`ShareMap::serialize`] are passed through.
    #[inline]
    pub fn serialize<S, K, V, Map>(
        value: &ShareMap<K, V, Map>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        ShareMap<K, V, Map>: serde::Serialize,
    {
        value.serialize(serializer)
    }

    /// Deserializes the data into a [`ShareMap`].
    ///
    /// # Errors
    ///
    /// Returns a [`serde::de::Error`] if the map contains duplicate keys.
    pub fn deserialize<'de, D, K, V, Map>(deserializer: D) -> Result<ShareMap<K, V, Map>, D::Error>
    where
        D: serde::Deserializer<'de>,
        K: Eq + Hash + serde::Deserialize<'de>,
        V: serde::Deserialize<'de>,
        Map: FromIterator<(K, usize)> + Len,
    {
        deserializer.deserialize_map(ShareMapVisitor(PhantomData))
    }

    #[derive(Debug)]
    struct ShareMapVisitor<K, V, Map>(PhantomData<ShareMap<K, V, Map>>);

    impl<'de, K, V, Map> serde::de::Visitor<'de> for ShareMapVisitor<K, V, Map>
    where
        K: Eq + Hash + serde::Deserialize<'de>,
        V: serde::Deserialize<'de>,
        Map: FromIterator<(K, usize)> + Len,
    {
        type Value = ShareMap<K, V, Map>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map with unique keys")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let mut entries = access.size_hint().unwrap_or(0).pipe(Vec::with_capacity);

            while let Some(entry) = access.next_entry()? {
                entries.push(entry);
            }

            ShareMap::try_from_iter(entries).map_err(serde::de::Error::custom)
        }
    }
}

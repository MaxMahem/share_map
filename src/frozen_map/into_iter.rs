use std::iter::FusedIterator;
use std::sync::Arc;

#[cfg(doc)]
use crate::FrozenMap;

/// An iterator over the key-value pairs in a [FrozenMap].
///
/// Order of iteration is dependent on the underlying map implementation.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use assert_unordered::*;
/// use swap_map::SwapMap;
/// use std::sync::Arc;
///
/// let snapshot = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?.snapshot();
/// let pairs: Vec<(&str, i32)> = Arc::into_inner(snapshot).ok_or("Multiple Owners")?.into_iter().collect();
/// assert_eq_unordered!(pairs, vec![("key1", 42), ("key2", 100)]);
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct IntoIter<K, V, Iter: Iterator<Item = (K, usize)>> {
    index_iter: Iter,
    store: Arc<[V]>,
}

impl<K, V, Iter> std::fmt::Debug for IntoIter<K, V, Iter>
where
    Iter: Iterator<Item = (K, usize)>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntoIter").finish_non_exhaustive()
    }
}

impl<K, V, Iter: Iterator<Item = (K, usize)>> IntoIter<K, V, Iter> {
    pub(crate) fn new(index_iter: Iter, store: Arc<[V]>) -> Self {
        Self { index_iter, store }
    }
}

impl<K, V: Clone, Iter> Iterator for IntoIter<K, V, Iter>
where
    Iter: Iterator<Item = (K, usize)>,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter
            .next()
            .and_then(|(key, index)| self.store.get(index).map(|val| (key, val.clone())))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.index_iter.size_hint()
    }
}

impl<K, V: Clone, Map> ExactSizeIterator for IntoIter<K, V, Map>
where
    Map: ExactSizeIterator<Item = (K, usize)>,
{
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}

impl<K, V: Clone, Iter> FusedIterator for IntoIter<K, V, Iter> where
    Iter: FusedIterator<Item = (K, usize)>
{
}

#[cfg(test)]
mod tests {
    use assert_unordered::assert_eq_unordered;

    use crate::SwapMap;
    use crate::UnitResultAny;

    #[test]
    fn test_into_iter() -> UnitResultAny {
        let iter = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?
            .into_snapshot()
            .ok_or("Multiple Owners")?
            .into_iter();

        let pairs: Vec<(i32, i32)> = iter.collect();

        assert_eq_unordered!(pairs, vec![(15, 42), (23, 100)]);
        Ok(())
    }

    #[test]
    fn test_into_iter_size_hint_len_fused_trait() -> UnitResultAny {
        let mut iter = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?
            .into_snapshot()
            .ok_or("Multiple Owners")?
            .into_iter();

        for len in (1..=2).rev() {
            assert_eq!(iter.len(), len);
            assert_eq!(iter.size_hint(), (len, Some(len)));

            iter.next();
        }

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None); // FusedIterator guarantees this remains None

        Ok(())
    }
}

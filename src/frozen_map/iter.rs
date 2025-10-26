use std::iter::FusedIterator;

/// A borrowed iterator over the key-value pairs in a [FrozenMap].
///
/// Order of iteration is dependent on the underlying map implementation.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use assert_unordered::*;
/// use swap_map::SwapMap;
///
/// let snapshot = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?.snapshot();
/// let pairs: Vec<(&i32, &i32)> = snapshot.iter().collect();
/// assert_eq_unordered!(pairs, vec![(&15, &42), (&23, &100)]);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Iter<'a, K: 'a, V, I>
where
    I: Iterator<Item = (&'a K, &'a usize)>,
{
    index_iter: I,
    store: &'a [V],
}

impl<'a, K, V, I> std::fmt::Debug for Iter<'a, K, V, I>
where
    I: Iterator<Item = (&'a K, &'a usize)>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter").finish_non_exhaustive()
    }
}

impl<'a, K, V, I> Iter<'a, K, V, I>
where
    I: Iterator<Item = (&'a K, &'a usize)>,
{
    pub(crate) fn new(index_iter: I, store: &'a [V]) -> Self {
        Self { index_iter, store }
    }
}

impl<'a, K, V, I> Iterator for Iter<'a, K, V, I>
where
    I: Iterator<Item = (&'a K, &'a usize)>,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter
            .next()
            .and_then(|(key, index)| self.store.get(*index).map(|val| (key, val)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.index_iter.size_hint()
    }
}

impl<'a, K, V: Clone, I> ExactSizeIterator for Iter<'a, K, V, I>
where
    I: ExactSizeIterator<Item = (&'a K, &'a usize)>,
{
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}

impl<'a, K, V: Clone, I> FusedIterator for Iter<'a, K, V, I> where
    I: FusedIterator<Item = (&'a K, &'a usize)>
{
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::SwapMap;
    use crate::UnitResultAny;

    #[test]
    fn test_borrow_iter() {
        let btree_map = BTreeMap::from([("key1", 42), ("key2", 100)]);
        let swap_map: SwapMap<&str, i32, BTreeMap<&str, usize>> = btree_map.clone().into();
        let snapshot = swap_map.snapshot();

        let swap_vec: Vec<_> = snapshot.iter().collect();
        let btree_vec: Vec<_> = btree_map.iter().collect();

        assert_eq!(swap_vec, btree_vec);
    }

    #[test]
    fn test_borrow_iter_size_hint_len_fused_trait() -> UnitResultAny {
        let snapshot = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?.snapshot();
        let mut iter = snapshot.iter();

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

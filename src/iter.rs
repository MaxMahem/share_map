use std::iter::FusedIterator;

#[cfg(doc)]
use crate::ShareMap;

/// A borrowed iterator over the key-value pairs in a [ShareMap].
///
/// Order of iteration is dependent on the underlying map implementation.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use assert_unordered::*;
/// use std::collections::BTreeMap;
/// use share_map::ShareMap;
///
/// // BTreeMap gurantees iteration order
/// let data_pairs = [(15, 42), (23, 100)];
/// let share_map = ShareMap::<_, _, BTreeMap<_, _>>::try_from_iter(data_pairs.clone())?;
/// let btree_map = BTreeMap::from(data_pairs.clone());
///
/// let share_pairs: Vec<_> = share_map.iter().collect();
/// let btree_pairs: Vec<_> = btree_map.iter().collect();
///
/// assert_eq!(share_pairs, btree_pairs);
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

    use crate::ShareMap;

    #[test]
    fn debug_is_expected() {
        let map = ShareMap::<_, _>::try_from_iter([(15, 42), (23, 100)]).expect("should be ok");
        let iter = map.iter();

        let debug = format!("{:?}", iter);

        assert_eq!(debug, "Iter { .. }");
    }

    #[test]
    fn borrow_iter_matches_btreemap() {
        let btree_map = BTreeMap::from([("key1", 42), ("key2", 100)]);
        let map: ShareMap<_, _, BTreeMap<_, _>> = btree_map.clone().into();

        let swap_vec: Vec<_> = map.iter().collect();
        let btree_vec: Vec<_> = btree_map.iter().collect();

        assert_eq!(swap_vec, btree_vec);
    }

    #[test]
    fn borrow_iter_size_hint_len_fused_trait_are_correct() {
        let map = ShareMap::<_, _>::try_from_iter([(15, 42), (23, 100)]).expect("should be ok");
        let mut iter = map.iter();

        for len in (1..=2).rev() {
            assert_eq!(iter.len(), len);
            assert_eq!(iter.size_hint(), (len, Some(len)));

            iter.next();
        }

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None); // FusedIterator guarantees this remains None
    }
}

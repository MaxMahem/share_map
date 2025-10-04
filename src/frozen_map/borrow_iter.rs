use std::{iter::FusedIterator, sync::Arc};

/// A borrowed iterator over the key-value pairs in a [FrozenMap].
///
/// Order of iteration is dependent on the underlying map implementation.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use assert_unordered::*;
/// use swap_map::SwapMap;
///
/// let snapshot = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?.snapshot();
/// let pairs: Vec<(&i32, &i32)> = snapshot.iter().collect();
/// assert_eq_unordered!(pairs, vec![(&15, &42), (&23, &100)]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BorrowIter<'a, K: 'a, V, Iter>
where
    Iter: Iterator<Item = (&'a K, &'a usize)>,
{
    index_iter: Iter,
    store: &'a Arc<[V]>,
}

impl<'a, K, V, Iter> BorrowIter<'a, K, V, Iter>
where
    Iter: Iterator<Item = (&'a K, &'a usize)>,
{
    pub(crate) fn new(index_iter: Iter, store: &'a Arc<[V]>) -> Self {
        Self { index_iter, store }
    }
}

impl<'a, K, V, Iter> Iterator for BorrowIter<'a, K, V, Iter>
where
    Iter: Iterator<Item = (&'a K, &'a usize)>,
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

impl<'a, K, V: Clone, Iter> ExactSizeIterator for BorrowIter<'a, K, V, Iter>
where
    Iter: ExactSizeIterator<Item = (&'a K, &'a usize)>,
{
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}

impl<'a, K, V: Clone, Iter> FusedIterator for BorrowIter<'a, K, V, Iter> where
    Iter: FusedIterator<Item = (&'a K, &'a usize)>
{
}

#[cfg(test)]
mod tests {
    use assert_unordered::assert_eq_unordered;

    use crate::SwapMap;
    use crate::UnitResultAny;

    #[test]
    fn test_borrow_iter() -> UnitResultAny {
        let snapshot = SwapMap::<i32, i32>::from_pairs([(15, 42), (23, 100)])?.snapshot();
        let iter = snapshot.iter();

        let pairs: Vec<(&i32, &i32)> = iter.collect();

        assert_eq_unordered!(pairs, vec![(&15, &42), (&23, &100)]);
        Ok(())
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

use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// A reference to a shared value.
#[derive(Clone)]
pub struct Handle<T> {
    store: Arc<[T]>,
    index: usize,
}

impl<T> Handle<T> {
    pub(crate) fn new(store: Arc<[T]>, index: usize) -> Self {
        debug_assert!(index < store.len());
        Self { store, index }
    }

    /// Returns `true` if the two referenced values are equal.
    ///
    /// This method provides a potentially faster path than the [`Eq`] trait. It first checks if
    /// the two [`Handle`]s are reference equal, if so, the values must be equal (they point to
    /// the same value and [Eq] implies values are equal to themselves), and so only checks
    /// equality of the values data themselves if the [`Handle`]s are not reference equal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{ShareMap, Handle};
    ///
    /// let share_map1 = ShareMap::<&str, i32>::try_from_iter([("key1", 42)])?;
    /// let share_map2 = ShareMap::<&str, i32>::try_from_iter([("key1", 42)])?;
    /// let eq_ref1 = share_map1.get_handle("key1").ok_or("Key not found")?;
    /// let eq_ref2 = share_map1.get_handle("key1").ok_or("Key not found")?;
    /// let eq_ref3 = share_map2.get_handle("key1").ok_or("Key not found")?;
    ///
    /// assert!(Handle::eq(&eq_ref1, &eq_ref2)); // equal by reference
    /// assert!(Handle::eq(&eq_ref1, &eq_ref3)); // equal by derefed value
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::should_implement_trait)] // we do implement Eq
    pub fn eq(this: &Handle<T>, other: &Handle<T>) -> bool
    where
        T: Eq,
    {
        Handle::ref_eq(this, other) || **this == **other
    }

    /// Returns `true` if the two referenced values are not equal.
    ///
    /// See [`Handle::eq`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{ShareMap, Handle};
    ///
    /// let share_map1 = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
    /// let share_map2 = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
    /// let ne_ref1 = share_map1.get_handle("key1").ok_or("Key not found")?;
    /// let ne_ref2 = share_map2.get_handle("key2").ok_or("Key not found")?;
    ///
    /// assert!(Handle::ne(&ne_ref1, &ne_ref2)); // not equal by reference
    /// # Ok(())
    /// # }
    /// ```
    pub fn ne(this: &Handle<T>, other: &Handle<T>) -> bool
    where
        T: Eq,
    {
        Handle::ref_ne(this, other) && **this != **other
    }

    /// Returns `true` if the two [`Handle`]s reference the same value instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use share_map::{ShareMap, Handle};
    ///
    /// let pairs = [("key1", 42)];
    /// let map1 = ShareMap::<_, _>::try_from_iter(pairs)?;
    /// let map2 = ShareMap::<_, _>::try_from_iter(pairs)?;
    ///
    /// // same Map, same key, equal
    /// let eq_handle1 = map1.get_handle("key1").ok_or("Key not found")?;
    /// let eq_handle2 = map1.get_handle("key1").ok_or("Key not found")?;
    /// assert!(Handle::ref_eq(&eq_handle1, &eq_handle2));
    ///
    /// // different Map, same value, not equal
    /// let ne_handle1 = map1.get_handle("key1").ok_or("Key not found")?;
    /// let ne_handle2 = map2.get_handle("key1").ok_or("Key not found")?;
    /// assert!(!Handle::ref_eq(&ne_handle1, &ne_handle2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn ref_eq(this: &Self, other: &Self) -> bool {
        std::ptr::eq(&**this, &**other)
    }

    /// Returns `true` if the two [`Handle`]s reference different value instances.
    ///
    /// See also [`Handle::ref_eq`].
    pub fn ref_ne(this: &Handle<T>, other: &Handle<T>) -> bool {
        !std::ptr::eq(&**this, &**other)
    }
}

impl<T> AsRef<T> for Handle<T> {
    fn as_ref(&self) -> &T {
        // Panic safety: `index` is guaranteed to be in bounds
        &self.store[self.index]
    }
}

impl<T> Borrow<T> for Handle<T> {
    fn borrow(&self) -> &T {
        // Panic safety: `index` is guaranteed to be in bounds
        &self.store[self.index]
    }
}

/// If `T` implements [Debug], [`Handle`] implements [Debug] by delegating to the derefed value.
impl<T: Debug> Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&**self, f)
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Panic safety: `index` is guaranteed to be in bounds
        &self.store[self.index]
    }
}

/// If `T` implements [`Display`], [`Handle`] implements [`Display`] by delegating to the derefed
/// value.
impl<T: Display> Display for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Display::fmt(&**self, f)
    }
}

/// If `T` implements [`Error`], [`Handle`] implements [`Error`] by delegating to the derefed value.
impl<T: Error> Error for Handle<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(&**self)
    }
}

impl<T: Hash> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

/// If `T` implements [Eq], [`Handle`] implements equality based on the derefed value.
impl<T: Eq> Eq for Handle<T> {}

/// If `T` implements [`PartialEq`], or [Eq], [`Handle`] implements equality based on the derefed
/// value. That is, two [`Handle`]s are equal if they derfed to the same value, even if they are
/// different references.
///
/// For Reference equality, see [`Handle::ref_eq`].
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use share_map::{ShareMap, Handle};
///
/// let map1 = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
/// let map2 = ShareMap::<_, _>::try_from_iter([("key1", 42), ("key2", 100)])?;
///
/// // different ShareMap, same value, equal
/// let eq_handle1 = map1.get("key1").ok_or("Key not found")?;
/// let eq_handle2 = map2.get("key1").ok_or("Key not found")?;
/// assert_eq!(eq_handle1, eq_handle2);
/// # Ok(())
/// # }
/// ```
impl<T: PartialEq> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

/// If `T` implements [PartialOrd], [`Handle`] implements comparison based on the derefed value.
impl<T: PartialOrd> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

/// If `T` implements [Ord], [`Handle`] implements comparison based on the derefed value.
impl<T: Ord> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::error::Error;
    use std::hash::BuildHasher;

    use crate::{Handle, ShareMap};

    #[test]
    fn deref_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        assert_eq!(*handle, value);
    }

    #[test]
    fn as_ref_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        assert_eq!(handle.as_ref(), &value);
    }

    #[test]
    fn borrow_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let borrow: &i32 = handle.borrow();

        assert_eq!(borrow, &value);
    }

    #[test]
    fn debug_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let handle_debug = format!("{:?}", handle);
        let value_debug = format!("{:?}", value);

        assert_eq!(handle_debug, value_debug);
    }

    #[test]
    fn display_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let handle_display = format!("{}", handle);
        let value_display = format!("{}", value);

        assert_eq!(handle_display, value_display);
    }

    #[test]
    fn cloned_value_matches() {
        let value = "hello world";

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let cloned_handle = handle.clone();

        assert_eq!(cloned_handle, handle);
        assert_eq!(*cloned_handle, *handle);
        assert!(Handle::eq(&cloned_handle, &handle));
    }

    #[test]
    fn cloned_handle_eq() {
        let value = "hello world";

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let cloned_handle = handle.clone();

        assert!(Handle::ref_eq(&cloned_handle, &handle));
    }

    #[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
    #[error("inner error")]
    struct InnerError;

    #[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
    #[error("test error")]
    struct TestError(#[source] InnerError);

    #[test]
    fn handle_delegates_source() {
        let error = TestError(InnerError);

        let handle = ShareMap::<_, _>::try_from_iter([("key1", error)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let handle_error_source_dbg = format!("{:?}", handle.source());
        let error_source_dbg = format!("{:?}", error.source());

        assert_eq!(handle_error_source_dbg, error_source_dbg);
    }

    #[test]
    fn eq_ne_same_ref() {
        let value = 42;

        let map = ShareMap::<_, _>::try_from_iter([("key1", value)]).expect("should be Ok");

        let handle1 = map.get_handle("key1").expect("should be Some");
        let handle2 = map.get_handle("key1").expect("should be Some");

        assert!(Handle::eq(&handle1, &handle2));
        assert!(!Handle::ne(&handle1, &handle2));
        assert_eq!(handle1, handle2);
        assert_eq!(handle1.cmp(&handle2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn eq_ne_different_ref_same_value() {
        let value = 42;

        let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");
        let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        assert!(Handle::eq(&handle1, &handle2));
        assert!(!Handle::ne(&handle1, &handle2));
        assert_eq!(handle1, handle2);
        assert_eq!(handle1.cmp(&handle2), std::cmp::Ordering::Equal);
    }

    #[test]
    fn ref_eq_ne_same_ref() {
        let value = 42;

        let map = ShareMap::<_, _>::try_from_iter([("key1", value)]).expect("should be Ok");

        let handle1 = map.get_handle("key1").expect("should be Some");
        let handle2 = map.get_handle("key1").expect("should be Some");

        assert!(Handle::ref_eq(&handle1, &handle2));
        assert!(!Handle::ref_ne(&handle1, &handle2));
    }

    #[test]
    fn ref_eq_different_ref_same_value() {
        let value = 42;

        let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");
        let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        assert!(!Handle::ref_eq(&handle1, &handle2));
        assert!(Handle::ref_ne(&handle1, &handle2));
    }

    #[test]
    fn hash_matches_value() {
        let value = 42;

        let handle = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let hasher = std::hash::RandomState::new();
        let value_hash = hasher.hash_one(value);
        let handle_hash = hasher.hash_one(handle);

        assert_eq!(handle_hash, value_hash);
    }

    #[test]
    fn partial_cmp_matches_value() {
        let value = 42.0;

        let handle1 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        let handle2 = ShareMap::<_, _>::try_from_iter([("key1", value)])
            .expect("should be Ok")
            .get_handle("key1")
            .expect("should be Some");

        assert_eq!(
            handle1.partial_cmp(&handle2),
            Some(std::cmp::Ordering::Equal)
        );
    }
}

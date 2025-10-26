use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

#[cfg(doc)]
use crate::SwapMap;

/// A reference to a value in a [SwapMap].
pub struct ValueRef<T> {
    store: Arc<[T]>,
    index: usize,
}

impl<T> ValueRef<T> {
    pub(crate) fn new(store: Arc<[T]>, index: usize) -> Self {
        debug_assert!(index < store.len());
        Self { store, index }
    }

    /// Returns `true` if the two referenced values are equal.
    ///
    /// This method first checks if the two [ValueRef]s are reference equal, if so, the values must
    /// be equal (they point to the same value and [Eq] implies reflexivity), and only checks
    /// equality of the derefed values if the [ValueRef]s are not reference equal.
    ///
    /// This method may be faster than equality via the [Eq] trait which relies only on
    /// dereferenced equality in all cases. Especially if equality for `T` is expensive or the
    /// values are likely to be reference equal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{SwapMap, ValueRef};
    ///
    /// let swap_map1 = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?;
    /// let swap_map2 = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?;
    /// let eq_ref1 = swap_map1.get("key1").ok_or("Key not found")?;
    /// let eq_ref2 = swap_map1.get("key1").ok_or("Key not found")?;
    /// let eq_ref3 = swap_map2.get("key1").ok_or("Key not found")?;
    ///
    /// assert!(ValueRef::eq(&eq_ref1, &eq_ref2)); // equal by reference
    /// assert!(ValueRef::eq(&eq_ref1, &eq_ref3)); // equal by derefed value
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::should_implement_trait)] // we do implement Eq
    pub fn eq(this: &ValueRef<T>, other: &ValueRef<T>) -> bool
    where
        T: Eq,
    {
        ValueRef::ref_eq(this, other) || **this == **other
    }

    /// Returns `true` if the two referenced values are not equal.
    ///
    /// See [ValueRef::eq]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{SwapMap, ValueRef};
    ///
    /// let swap_map1 = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let swap_map2 = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
    /// let ne_ref1 = swap_map1.get("key1").ok_or("Key not found")?;
    /// let ne_ref2 = swap_map2.get("key2").ok_or("Key not found")?;
    ///
    /// assert!(ValueRef::ne(&ne_ref1, &ne_ref2)); // not equal by reference
    /// # Ok(())
    /// # }
    /// ```
    pub fn ne(this: &ValueRef<T>, other: &ValueRef<T>) -> bool
    where
        T: Eq,
    {
        ValueRef::ref_ne(this, other) || **this != **other
    }

    /// Returns `true` if the two [ValueRef]s reference the same location in the same [SwapMap].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use swap_map::{SwapMap, ValueRef};
    ///
    /// let swap_map1 = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?;
    /// let swap_map2 = SwapMap::<&str, i32>::from_pairs([("key1", 42)])?;
    ///
    /// // same SwapMap, same key, equal
    /// let eq_value_ref1 = swap_map1.get("key1").ok_or("Key not found")?;
    /// let eq_value_ref2 = swap_map1.get("key1").ok_or("Key not found")?;
    /// assert!(ValueRef::ref_eq(&eq_value_ref1, &eq_value_ref2));
    ///
    /// // different SwapMap, same value, not equal
    /// let ne_value_ref1 = swap_map1.get("key1").ok_or("Key not found")?;
    /// let ne_value_ref2 = swap_map2.get("key1").ok_or("Key not found")?;
    /// assert!(!ValueRef::ref_eq(&ne_value_ref1, &ne_value_ref2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn ref_eq(this: &ValueRef<T>, other: &ValueRef<T>) -> bool {
        Arc::ptr_eq(&this.store, &other.store) && this.index == other.index
    }

    /// Returns `true` if the two [ValueRef]s reference different [SwapMap]s, or different
    /// locations in the same [SwapMap].
    ///
    /// See also [ValueRef::ref_eq].
    pub fn ref_ne(this: &ValueRef<T>, other: &ValueRef<T>) -> bool {
        !Arc::ptr_eq(&this.store, &other.store) || this.index != other.index
    }
}

impl<T> AsRef<T> for ValueRef<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for ValueRef<T> {
    fn borrow(&self) -> &T {
        self
    }
}

/// Clones the [ValueRef], returning a new reference to the same value. The referenced value is not
/// cloned.
impl<T> Clone for ValueRef<T> {
    fn clone(&self) -> Self {
        Self::new(self.store.clone(), self.index)
    }
}

/// If `T` implements [Debug], [ValueRef] implements [Debug] by delegating to the derefed value.
impl<T: Debug> Debug for ValueRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Debug::fmt(&**self, f)
    }
}

impl<T> Deref for ValueRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Panic safety: `index` is guaranteed to be in bounds
        &self.store[self.index]
    }
}

/// If `T` implements [Display], [ValueRef] implements [Display] by delegating to the derefed
/// value.
impl<T: Display> Display for ValueRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        Display::fmt(&**self, f)
    }
}

/// If `T` implements [Error], [ValueRef] implements [Error] by delegating to the derefed value.
impl<T: Error> Error for ValueRef<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(&**self)
    }
}

impl<T: Hash> Hash for ValueRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

/// If `T` implements [Eq], [ValueRef] implements equality based on the derefed value.
///
///
impl<T: Eq> Eq for ValueRef<T> {}

/// If `T` implements [PartialEq], or [Eq], [ValueRef] implements equality based on the derefed
/// value. That is, two [ValueRef]s are equal if they derfed to the same value, even if they are
/// different references.
///
/// For Reference equality, see [ValueRef::ref_eq].
///
/// # Examples
///
/// ```rust
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use swap_map::{SwapMap, ValueRef};
///
/// let swap_map1 = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
/// let swap_map2 = SwapMap::<&str, i32>::from_pairs([("key1", 42), ("key2", 100)])?;
///
/// // different SwapMap, same value, equal
/// let eq_value_ref1 = swap_map1.get("key1").ok_or("Key not found")?;
/// let eq_value_ref2 = swap_map2.get("key1").ok_or("Key not found")?;
/// assert_eq!(eq_value_ref1, eq_value_ref2);
/// # Ok(())
/// # }
/// ```
impl<T: PartialEq> PartialEq for ValueRef<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

/// If `T` implements [PartialOrd], [ValueRef] implements comparison based on the derefed value.
impl<T: PartialOrd> PartialOrd for ValueRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

/// If `T` implements [Ord], [ValueRef] implements comparison based on the derefed value.
impl<T: Ord> Ord for ValueRef<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

#[cfg(test)]
mod tests {
    use crate::SwapMap;

    #[test]
    fn deref() -> Result<(), Box<dyn std::error::Error>> {
        let map: SwapMap<&str, i32> = SwapMap::from_pairs([("key1", 42)])?;
        let value_ref = map.get("key1").ok_or("key not found")?;

        assert_eq!(*value_ref, 42);

        Ok(())
    }

    #[test]
    fn debug() -> Result<(), Box<dyn std::error::Error>> {
        let map: SwapMap<&str, i32> = SwapMap::from_pairs([("key1", 42)])?;
        let value_ref = map.get("key1").ok_or("key not found")?;

        let debug_str = format!("{:?}", value_ref);

        assert_eq!(debug_str, "42");

        Ok(())
    }
}

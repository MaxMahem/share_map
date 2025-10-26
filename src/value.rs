use std::{ops::Deref, sync::Arc};

/// An enum representing the ownership of a value.
///
/// A safer/more ergonic way of representing the result of [Arc::try_unwrap], as it does not allow
/// problematic [Result] methods from chaining.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use swap_map::Value;
///
/// let arc = Arc::new("Hello");
/// let value: Value<_> = Arc::try_unwrap(arc).into();
/// assert!(value.is_owned());
///
/// let arc = Arc::new("World");
/// let value: Value<_> = Arc::try_unwrap(arc.clone()).into();
/// assert!(value.is_shared());
/// ```
#[derive(Debug, derive_more::IsVariant)]
pub enum Value<T> {
    /// The value is wholely owned.
    Owned(T),
    /// The value's ownership is shared.
    Shared(Arc<T>),
}

impl<T> Value<T> {
    /// Consumes the value and returns it directly if owned, or clones it if shared.
    ///
    /// Analogous to [Arc::unwrap_or_clone].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use swap_map::Value;
    ///
    /// let value = Value::Owned("World");
    /// let owned: &str = Value::into_owned_or_clone(value);
    /// assert_eq!(owned, "World");
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc);
    /// let unwrapped: &str = Value::into_owned_or_clone(value);
    /// assert_eq!(unwrapped, "Hello");
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc.clone());
    /// let cloned: &str = Value::into_owned_or_clone(value);
    /// assert_eq!(cloned, "Hello");
    /// ```
    pub fn into_owned_or_clone(value: Value<T>) -> T
    where
        T: Clone,
    {
        match value {
            Value::Owned(value) => value,
            Value::Shared(value) => Arc::unwrap_or_clone(value),
        }
    }

    /// Tries to convert the value into an owned value by unwrapping a shared arc.
    ///
    /// Analogous to [Arc::try_unwrap]. If the value is already owned, nothing is done.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use swap_map::Value;
    ///
    /// let value = Value::Owned("World");
    /// let owned = Value::try_into_owned(value);
    /// assert!(owned.is_owned());
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc);
    /// let unwrapped = Value::try_into_owned(value);
    /// assert!(unwrapped.is_owned());
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc.clone());
    /// let shared = Value::try_into_owned(value);
    /// assert!(shared.is_shared());
    /// ```
    pub fn try_into_owned(value: Value<T>) -> Value<T> {
        match value {
            owned @ Value::Owned(_) => owned,
            Value::Shared(value) => Arc::try_unwrap(value).into(),
        }
    }

    /// Consumes the value and returns it directly if solely owned, dropping the arc otherwise.
    ///
    /// Analogous to [Arc::into_inner].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use swap_map::Value;
    ///
    /// let value = Value::Owned("World");
    /// let owned = Value::into_owned(value);
    /// assert!(owned.is_some());
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc);
    /// let unwrapped = Value::into_owned(value);
    /// assert!(unwrapped.is_some());
    ///
    /// let arc = Arc::new("Hello");
    /// let value = Value::Shared(arc.clone());
    /// let shared = Value::into_owned(value);
    /// assert!(shared.is_none());
    /// ```
    pub fn into_owned(value: Value<T>) -> Option<T> {
        match value {
            Value::Owned(value) => Some(value),
            Value::Shared(value) => Arc::into_inner(value),
        }
    }
}

impl<T> From<Arc<T>> for Value<T> {
    fn from(value: Arc<T>) -> Self {
        Arc::try_unwrap(value).into()
    }
}

impl<T> From<Result<T, Arc<T>>> for Value<T> {
    fn from(value: Result<T, Arc<T>>) -> Self {
        match value {
            Ok(value) => Value::Owned(value),
            Err(value) => Value::Shared(value),
        }
    }
}

impl<T> AsRef<T> for Value<T> {
    fn as_ref(&self) -> &T {
        match self {
            Value::Owned(value) => value,
            Value::Shared(value) => value,
        }
    }
}

impl<T> Deref for Value<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

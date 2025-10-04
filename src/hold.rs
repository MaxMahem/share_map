use std::{ops::Deref, sync::Arc};

/// An enum representing the ownership of a value.
#[derive(Debug, derive_more::IsVariant)]
pub enum Hold<T> {
    /// The value is owned by the enum.
    Owned(T),
    /// The value is shared by the enum.
    Shared(Arc<T>),
}

impl<T> From<Result<T, Arc<T>>> for Hold<T> {
    fn from(value: Result<T, Arc<T>>) -> Self {
        match value {
            Ok(value) => Hold::Owned(value),
            Err(value) => Hold::Shared(value),
        }
    }
}

impl<T> AsRef<T> for Hold<T> {
    fn as_ref(&self) -> &T {
        match self {
            Hold::Owned(value) => value,
            Hold::Shared(value) => value,
        }
    }
}

impl<T> Deref for Hold<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

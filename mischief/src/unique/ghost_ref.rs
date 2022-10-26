use ::core::{marker::PhantomData, mem::forget};

use crate::unique::{Singleton, Unique};

/// A "ghost" reference that is guaranteed to be zero-sized and obey borrowing
/// and ownership semantics
pub struct GhostRef<T>(PhantomData<T>);

impl<T: Clone> Clone for GhostRef<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T: Copy> Copy for GhostRef<T> {}

impl<T> GhostRef<T> {
    /// Returns a new `GhostRef` from the given value.
    ///
    /// Note that the given value will be leaked forever. Leaking types that
    /// have a meaningful `Drop` implementation can cause system resources to
    /// leak.
    pub fn leak(value: T) -> Self {
        forget(value);
        Self(PhantomData)
    }
}

// SAFETY: Because `GhostRef` can only be constructed by leaking a `T`, and `T`
// is guaranteed to be `Unique`, the `GhostRef` is also `Unique`.
unsafe impl<T: Unique> Unique for GhostRef<T> {}

// SAFETY: Because `GhostRef` can only be constructed by leaking a `T`, and `T`
// is guaranteed to be a `Singleton`, the `GhostRef` is also a `Singleton`.
unsafe impl<T: Singleton> Singleton for GhostRef<T> {}

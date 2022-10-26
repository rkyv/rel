/// A type that points to a single location in memory.
///
/// # Safety
///
/// If the type implementing `Pointer` also implements `Deref` or `DerefMut`,
/// the pointer returned by `Target` must be equal to the references returned
/// from `deref` and `deref_mut`.
pub unsafe trait Pointer {
    /// The target value of this type.
    type Target: ?Sized;

    /// Returns a pointer to this type's target.
    fn target(&self) -> *mut Self::Target;
}

// SAFETY: `*const T` does not implement `Deref` or `DerefMut` and never will
// because `*const T` is a fundamental type.
unsafe impl<T: ?Sized> Pointer for *const T {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.cast_mut()
    }
}

// SAFETY: `*mut T` does not implement `Deref` or `DerefMut` and never will
// because `*mut T` is a fundamental type.
unsafe impl<T: ?Sized> Pointer for *mut T {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        *self
    }
}

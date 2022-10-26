use ::core::{fmt, marker::PhantomData, ops::Deref, ptr::NonNull};
use ::mischief::{
    Pointer,
    Region,
    RestructurablePointer,
    Singleton,
    Unique,
    Within,
};
use ::munge::{Destructure, Restructure};

use crate::{
    fmt::{DebugRaw, DisplayRaw},
    Pinned,
};

/// An immutable reference, like `&T`.
///
/// Internally, the reference is stored as a pointer to avoid provenance
/// narrowing.
pub struct Ref<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Clone for Ref<'a, T> {
    fn clone(&self) -> Self {
        // SAFETY: `self.as_ptr()` is the internal pointer of this `Ref`, and
        // upholds all of the safety requirements of `Ref::new_unchecked`.
        unsafe { Ref::new_unchecked(self.as_ptr()) }
    }
}

impl<'a, T: ?Sized> Copy for Ref<'a, T> {}

impl<'a, T: ?Sized> Ref<'a, T> {
    /// Creates a new `Ref` from a shared pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned, and valid for reads.
    /// - `ptr` must not alias any other mutable references for `'a`.
    /// - The value pointed to by `ptr` must be initialized.
    pub unsafe fn new_unchecked(ptr: *const T) -> Self {
        Self {
            // SAFETY: The caller has guaranteed that `ptr` is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr.cast_mut()) },
            _phantom: PhantomData,
        }
    }

    /// Returns a pointer to the referenced value.
    pub fn as_ptr(self) -> *mut T {
        self.ptr.as_ptr()
    }
}

// SAFETY: `Ref` returns the same value from `target` and `deref`.
unsafe impl<T: ?Sized> Pointer for Ref<'_, T> {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.ptr.as_ptr()
    }
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `Ref<'a, T>` lives for `'a` at most, the lifetime of
        //   `&self` must be shorter than `'a`. That lifetime is used for the
        //   returned reference, so the returned reference is valid for `'a` and
        //   has shared read-only aliasing.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: DebugRaw + ?Sized> fmt::Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugRaw::fmt_raw(*self, f)
    }
}

impl<T: DisplayRaw + ?Sized> fmt::Display for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayRaw::fmt_raw(*self, f)
    }
}

// SAFETY: `Destructure::underlying` for `Ref` returns the same pointer as
// `Pointer::target`.
unsafe impl<T: ?Sized> RestructurablePointer for Ref<'_, T> {}

// SAFETY: `T` is only located in `R`, so the targets of all `Ref<'_, T>` must
// be located in `R`.
unsafe impl<T: Pinned<R> + ?Sized, R: Region> Within<R> for Ref<'_, T> {}

// SAFETY:
// - `Ref<'a, T>` is destructured by reference, so its `Destructuring` type is
//   `Ref`.
// - `underlying` returns the pointer inside the `Ref<'a, T>`, which is
//   guaranteed to be non-null, properly-aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for Ref<'a, T> {
    type Underlying = T;
    type Destructuring = ::munge::Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `Ref<'a, U>` that borrows the restructured
// field because `Ref<'a, T>` is destructured by reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for Ref<'a, T> {
    type Restructured = Ref<'a, U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY:
        // - A pointer to a subfield of a `Ref` is also non-null, properly
        //   aligned, and valid for reads and writes.
        // - `munge` enforces that the field pointer cannot alias another
        //   accessible reference to the field. Because `Ref` is a shared borrow
        //   of the entire object, there cannot be another mutable reference to
        //   one of its fields.
        // - All of the fields of a `Ref` must be initialized and immovable
        //   because the overall `Ref` is initialized and immovable.
        unsafe { Ref::new_unchecked(ptr) }
    }
}

// SAFETY: Because the borrowed `T` is unique and shared references borrow their
// state from the underlying `T`, all `Ref<T>` to unique `T` must be sharing the
// same value.
unsafe impl<T: Unique + ?Sized> Singleton for Ref<'_, T> {}

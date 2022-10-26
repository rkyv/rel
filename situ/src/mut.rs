use ::core::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use ::mischief::{Pointer, Region, RestructurablePointer, Unique, Within};
use ::munge::{Destructure, Restructure};

use crate::{
    fmt::{DebugRaw, DisplayRaw},
    DropRaw,
    Pinned,
    Ref,
    Val,
};

/// An immovable mutable reference, like `&mut T` that cannot be relocated.
pub struct Mut<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Mut<'a, T> {
    /// Creates a new `Mut` from an exclusively borrowed pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned, and valid for reads and
    ///   writes.
    /// - `ptr` must not alias any other accessible references for `'a`.
    /// - The value pointed to by `ptr` must be initialized and immovable.
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            // SAFETY: The caller has guaranteed that `ptr` is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _phantom: PhantomData,
        }
    }

    /// Constructs a new `Mut` by mapping the interior pointer.
    ///
    /// # Safety
    ///
    /// - The pointer returned by `f` must be non-null, properly aligned, and
    ///   valid for reads and writes.
    /// - The pointer returned by `f` must not alias any other accessible
    ///   references for `'a`.
    /// - The value pointed to by the pointer returned by `f` must be
    ///   initialized and immutable.
    pub unsafe fn map_unchecked<F, U>(self, f: F) -> Mut<'a, U>
    where
        F: FnOnce(*mut T) -> *mut U,
    {
        // SAFETY: The caller has guaranteed that pointer returned by `f` meets
        // all of the safety requirements of `new_unchecked`.
        unsafe { Mut::new_unchecked(f(self.as_ptr())) }
    }

    /// Returns a pointer to the referenced value.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Returns a `NonNull` to the referenced value.
    pub fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    /// Returns a `Ref` of the referenced value.
    pub fn as_ref(&self) -> Ref<'_, T> {
        // SAFETY: The requirements for `Ref` are a subset of those for `Mut`.
        unsafe { Ref::new_unchecked(self.as_ptr()) }
    }

    /// Returns a reborrowed `Mut` of the referenced value.
    pub fn as_mut(&mut self) -> Mut<'_, T> {
        // SAFETY: The reborrowed `Mut` lives shorter than `self` and satisfies
        // all of the same requirements.
        unsafe { Mut::new_unchecked(self.as_ptr()) }
    }

    /// Assumes ownership of the value in the `Mut`.
    ///
    /// # Safety
    ///
    /// - The value pointed to by the `Mut` must be valid for dropping.
    /// - The caller must ensure that the value pointed to by the `Mut` is never
    ///   accessed in an illegal state. Because `Val` may drop the value, care
    ///   must be taken to forget the `Val` or replace the value after dropping
    ///   it.
    pub unsafe fn take(self) -> Val<'a, T>
    where
        T: DropRaw,
    {
        // SAFETY: `Mut` upholds all of the invariants required for `Val`,
        // except that the caller has guaranteed that the value pointed to by
        // `self` is valid for dropping.
        unsafe { Val::new_unchecked(self.as_ptr()) }
    }
}

// SAFETY: `Mut` returns the same value from `target`, `deref`, and `deref_mut`.
unsafe impl<T: ?Sized> Pointer for Mut<'_, T> {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.ptr.as_ptr()
    }
}

// SAFETY: `T` is only located in `R`, so the targets of all `Mut<'_, T>` must
// be located in `R`.
unsafe impl<T: Pinned<R> + ?Sized, R: Region> Within<R> for Mut<'_, T> {}

impl<T: ?Sized> Deref for Mut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `Mut<'a, T>` lives for `'a` at most, the lifetime of
        //   `&self` must be shorter than `'a`. That lifetime is used for the
        //   returned reference, so the returned reference is valid for `'a` and
        //   has shared read-only aliasing.
        unsafe { self.ptr.as_ref() }
    }
}

// Note that `T` must be `Unpin` to avoid violating the immovability invariant
// of `Mut`.
impl<T: Unpin + ?Sized> DerefMut for Mut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `Mut<'a, T>` lives for `'a` at most, the lifetime of
        //   `&mut self` must be shorter than `'a`. That lifetime is used for
        //   the returned reference, so the returned reference is valid for `'a`
        //   and has unique read-write aliasing.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: DebugRaw + ?Sized> fmt::Debug for Mut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugRaw::fmt_raw(self.as_ref(), f)
    }
}

impl<T: DisplayRaw + ?Sized> fmt::Display for Mut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayRaw::fmt_raw(self.as_ref(), f)
    }
}

// SAFETY: `Destructure::underlying` for `Mut` returns the same pointer as
// `Pointer::target`.
unsafe impl<T: ?Sized> RestructurablePointer for Mut<'_, T> {}

// SAFETY:
// - `Mut<'a, T>` is destructured by reference, so its `Destructuring` type is
//   `Ref`.
// - `underlying` returns the pointer inside the `Mut<'a, T>`, which is
//   guaranteed to be non-null, properly-aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for Mut<'a, T> {
    type Underlying = T;
    type Destructuring = ::munge::Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `Mut<'a, U>` that borrows the restructured
// field because `Mut<'a, T>` is destructured by reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for Mut<'a, T> {
    type Restructured = Mut<'a, U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY:
        // - A pointer to a subfield of a `Mut` is also non-null, properly
        //   aligned, and valid for reads and writes.
        // - `munge` enforces that the field pointer cannot alias another
        //   accessible reference to the field. Because `Mut` is an exclusive
        //   borrow of the entire object, there cannot be another reference to
        //   one of its fields.
        // - All of the fields of a `Mut` must be initialized and immovable
        //   because the overall `Mut` is initialized and immovable.
        unsafe { Mut::new_unchecked(ptr) }
    }
}

// SAFETY: Because the borrowed `T` is unique and mutable references are
// exclusive, there can only ever be one `Mut` to each unique `T` at any time.
unsafe impl<T: Unique + ?Sized> Unique for Mut<'_, T> {}

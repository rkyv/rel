//! Variants of `ops` traits that work with raw references.

use ::ptr_meta::metadata;

use crate::{Mut, Ref};

/// A variant of `Deref` that works with raw references.
pub trait DerefRaw {
    /// The resulting type after dereferencing.
    type Target: ?Sized;

    /// Dereferences the value.
    fn deref_raw(this: Ref<'_, Self>) -> Ref<'_, Self::Target>;
}

/// A variant of `DerefMut` that works with raw references.
pub trait DerefMutRaw: DerefRaw {
    /// Mutably dereferences the value.
    fn deref_mut_raw(this: Mut<'_, Self>) -> Mut<'_, Self::Target>;
}

/// A variant of `Index` that works with raw references.
pub trait IndexRaw<Idx> {
    /// The returned type after indexing.
    type Output: ?Sized;

    /// Performs the indexing (`container[index]`) operation.
    ///
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    fn index_raw(this: Ref<'_, Self>, index: Idx) -> Ref<'_, Self::Output>;

    /// Performs the indexing (`container[index]`) operation without performing
    /// bounds checking
    ///
    /// # Safety
    ///
    /// `index` must be in bounds for `this`.
    unsafe fn index_raw_unchecked(
        this: Ref<'_, Self>,
        index: Idx,
    ) -> Ref<'_, Self::Output>;
}

/// A variant of `IndexMut` that works with raw references.
pub trait IndexMutRaw<Idx>: IndexRaw<Idx> {
    /// Performs the mutable indexing (`container[index]`) operation.
    ///
    /// # Panics
    ///
    /// May panic if the index is out of bounds.
    fn index_mut_raw(this: Mut<'_, Self>, index: Idx) -> Mut<'_, Self::Output>;

    /// Performs the mutable indexing (`container[index]`) operation without
    /// performing bounds checking
    ///
    /// # Safety
    ///
    /// `index` must be in bounds for `this`.
    unsafe fn index_mut_raw_unchecked(
        this: Mut<'_, Self>,
        index: Idx,
    ) -> Mut<'_, Self::Output>;
}

impl<T, const N: usize> IndexRaw<usize> for [T; N] {
    type Output = T;

    fn index_raw(this: Ref<'_, Self>, index: usize) -> Ref<'_, Self::Output> {
        assert!(index < N);
        // SAFETY: We asserted that `index` is less than `N` and so it in
        // bounds.
        unsafe { Self::index_raw_unchecked(this, index) }
    }

    unsafe fn index_raw_unchecked(
        this: Ref<'_, Self>,
        index: usize,
    ) -> Ref<'_, Self::Output> {
        // SAFETY: This pointer add is safe because the pointer is guaranteed to
        // be to the first `T` in an array of `N` consecutive `T`, and the
        // resulting pointer must be in-bounds because the caller has guaranteed
        // that `index` is less than `N`.
        let ptr = unsafe { this.as_ptr().cast::<T>().add(index) };
        // SAFETY: The offset pointer is to an element of the original array,
        // and so must be non-null, properly aligned, valid for reads, and
        // initialized. It has the same shared aliasing as the `Ref` it is
        // derived from, and so must not alias any other mutable references for
        // `'_`.
        unsafe { Ref::new_unchecked(ptr) }
    }
}

impl<T, const N: usize> IndexMutRaw<usize> for [T; N] {
    fn index_mut_raw(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        assert!(index < N);
        // SAFETY: We asserted that `index` is less than `N` and so is in
        // bounds.
        unsafe { Self::index_mut_raw_unchecked(this, index) }
    }

    unsafe fn index_mut_raw_unchecked(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        // SAFETY: This pointer add is safe because the pointer is guaranteed to
        // be to the first `T` in an array of `N` consecutive `T`, and the
        // resulting pointer must be in-bounds because the caller has guaranteed
        // that `index` is less than `N`.
        let ptr = unsafe { this.as_ptr().cast::<T>().add(index) };
        // SAFETY: The offset pointer is to an element of the original array,
        // and so must be non-null, properly aligned, valid for reads, and
        // initialized. It has the same mutable aliasing as the `Mut` it is
        // derived from, and so must not alias any other accessible references
        // for `'_`.
        unsafe { Mut::new_unchecked(ptr) }
    }
}

impl<T> IndexRaw<usize> for [T] {
    type Output = T;

    fn index_raw(this: Ref<'_, Self>, index: usize) -> Ref<'_, Self::Output> {
        let slice_ptr = this.as_ptr();
        let len = metadata(slice_ptr);
        assert!(index < len, "index out of bounds");
        // SAFETY: We asserted that `index` is less than `len` and so it in
        // bounds.
        unsafe { Self::index_raw_unchecked(this, index) }
    }

    unsafe fn index_raw_unchecked(
        this: Ref<'_, Self>,
        index: usize,
    ) -> Ref<'_, Self::Output> {
        let slice_ptr = this.as_ptr();
        // SAFETY: This pointer add is safe because the pointer is guaranteed to
        // be to the first `T` in a slice of `len` consecutive `T`, and the
        // resulting pointer must be in-bounds because the caller has guaranteed
        // that `index` is less than `len`.
        let ptr = unsafe { slice_ptr.cast::<T>().add(index) };
        // SAFETY: The offset pointer is to an element of the original slice,
        // and so must be non-null, properly aligned, valid for reads, and
        // initialized. It has the same shared aliasing as the `Ref` it is
        // derived from, and so must not alias any other mutable references for
        // `'_`.
        unsafe { Ref::new_unchecked(ptr) }
    }
}

impl<T> IndexMutRaw<usize> for [T] {
    fn index_mut_raw(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        let slice_ptr = this.as_ptr();
        let len = metadata(slice_ptr);
        assert!(index < len, "index out of bounds");
        // SAFETY: We asserted that `index` is less than `len` and so is in
        // bounds.
        unsafe { Self::index_mut_raw_unchecked(this, index) }
    }

    unsafe fn index_mut_raw_unchecked(
        this: Mut<'_, Self>,
        index: usize,
    ) -> Mut<'_, Self::Output> {
        let slice_ptr = this.as_ptr();
        // SAFETY: This pointer add is safe because the pointer is guaranteed to
        // be to the first `T` in a slice of `len` consecutive `T`, and the
        // resulting pointer must be in-bounds because the caller has guaranteed
        // that `index` is less than `len`.
        let ptr = unsafe { slice_ptr.cast::<T>().add(index) };
        // SAFETY: The offset pointer is to an element of the original slice,
        // and so must be non-null, properly aligned, valid for reads, and
        // initialized. It has the same mutable aliasing as the `Mut` it is
        // derived from, and so must not alias any other accessible references
        // for `'_`.
        unsafe { Mut::new_unchecked(ptr) }
    }
}

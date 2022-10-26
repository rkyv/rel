use ::core::{
    alloc::Layout,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{slice_from_raw_parts_mut, NonNull},
};
use ::munge::{Destructure, Restructure};
use ::ptr_meta::Pointee;

use crate::{layout_of_val_raw, Metadata, Pointer, RestructurablePointer};

/// A memory location that may or may not have a value initialized in it.
pub struct Slot<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> Slot<'a, T> {
    /// Returns the layout of the pointer held in the slot.
    pub fn pointer_layout(&self) -> Layout
    where
        T: Pointee,
        T::Metadata: Metadata<T>,
    {
        layout_of_val_raw(self.as_ptr())
    }

    /// Creates a new slot from an exclusive pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned, and valid for reads and
    ///   writes.
    /// - `ptr` must not alias any other accessible references for `'a`.
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            // SAFETY: `ptr` is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _phantom: PhantomData,
        }
    }

    /// Gets a shared reference to the value in the slot.
    ///
    /// # Safety
    ///
    /// The contents of the slot must be initialized. See `assume_init` for
    /// details.
    pub unsafe fn assume_init_ref(&self) -> &'a T {
        // SAFETY:
        // - `self.as_ptr()` is always non-null, properly aligned, and valid for
        //   reads and writes.
        // - `self.as_ptr()` has the same aliasing as `self`, which is a shared
        //   reference.
        // - The caller has guaranteed that the value in this slot is
        //   initialized.
        unsafe { &*self.as_ptr() }
    }

    /// Gets a mutable reference to the value in the slot.
    ///
    /// # Safety
    ///
    /// The contents of the slot must be initialized. See `assume_init` for
    /// details.
    pub unsafe fn assume_init_mut(&mut self) -> &'a mut T {
        // SAFETY:
        // - `self.as_ptr()` is always non-null, properly aligned, and valid for
        //   reads and writes.
        // - `self.as_ptr()` has the same aliasing as `self`, which is a mutable
        //   reference.
        // - The caller has guaranteed that the value in this slot is
        //   initialized.
        unsafe { &mut *self.as_ptr() }
    }

    /// Drops the value in the slot.
    ///
    /// # Safety
    ///
    /// The contents of the slot must be initialized. See `assume_init` for
    /// details.
    pub unsafe fn assume_init_drop(&mut self) {
        // SAFETY:
        // - `self.as_ptr()` is always non-null, properly aligned, and valid for
        //   reads and writes.
        // - `self.as_ptr()` has the same aliasing as `self`, which is a mutable
        //   reference.
        // - The caller has guaranteed that the value in this slot is
        //   initialized.
        unsafe { self.as_ptr().drop_in_place() }
    }

    /// Gets a pointer to the underlying memory.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Gets a mutable borrow from this slot.
    pub fn as_mut<'b>(&'b mut self) -> Slot<'b, T>
    where
        'a: 'b,
    {
        // SAFETY:
        // - `self.ptr` is always non-null, properly aligned, and valid for
        //   reads and writes.
        // - `self.ptr` never aliases any other accessible references.
        Self {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

    /// Writes zeroes to every byte of the backing memory.
    pub fn zero(&mut self)
    where
        T: Pointee,
        T::Metadata: Metadata<T>,
    {
        // SAFETY: `self.ptr` is always properly aligned and valid for writes of
        // `layout.size()` bytes.
        unsafe {
            self.as_ptr()
                .cast::<u8>()
                .write_bytes(0, self.pointer_layout().size());
        }
    }

    /// Returns a slot of the underlying bytes.
    pub fn as_bytes(self) -> Slot<'a, [u8]>
    where
        T: Pointee,
        T::Metadata: Metadata<T>,
    {
        let layout = self.pointer_layout();
        let slice_ptr =
            slice_from_raw_parts_mut(self.as_ptr().cast::<u8>(), layout.size());
        // SAFETY:
        // - `slice_ptr` is non-null because `self.as_ptr()` is always non-null.
        // - `slice_ptr` points to `layout.size()` bytes starting at
        //   `self.as_ptr()`, all of which belong to the slot and so are valid
        //   for reads and writes.
        // - `slice_ptr` points to a slice of `u8`, so it is always properly
        //   aligned.
        // - `slice_ptr` does not alias any other accessible references for `'a`
        //   because `self` does not alias any accessible references for `'a`.
        unsafe { Slot::new_unchecked(slice_ptr) }
    }

    /// Attempts to cast the type of the `Slot` from `T` to `U`.
    ///
    /// Returns `None` if `U` requires a higher alignment than the pointer in
    /// `self` or `U` is larger than `T`.
    pub fn cast_checked<U>(self) -> Option<Slot<'a, U>>
    where
        T: Pointee,
        T::Metadata: Metadata<T>,
    {
        use core::mem::{align_of, size_of};

        let size = self.pointer_layout().size();
        // TODO strict_provenance: Use `pointer.addr()`.
        #[allow(clippy::as_conversions)]
        let is_aligned =
            self.as_ptr().cast::<u8>() as usize & (align_of::<U>() - 1) == 0;
        if size >= size_of::<U>() && is_aligned {
            // SAFETY: `self` points to enough memory for a `U`, and that memory
            // also has suitable alignment for a `U`.
            Some(unsafe { self.cast() })
        } else {
            None
        }
    }

    /// Casts the type of the `Slot` from `T` to `U`.
    ///
    /// # Safety
    ///
    /// The slot must point to memory suitable for holding a `U`. In addition to
    /// being the right size and aligment, the caller must also guarantee that
    /// the slot pointer will not alias any other accessible references after
    /// being cast.
    pub unsafe fn cast<U>(self) -> Slot<'a, U> {
        // SAFETY:
        // - `self.ptr` is a `NonNull` and so is guaranteed to be non-null. The
        //   pointer of a `Slot` is always properly aligned and valid for reads
        //   and writes.
        // - `self.ptr` is guaranteed not to alias any other accessible
        //   references for `'a`.
        // - The caller has guaranteed that `self.ptr` is suitable for holding a
        //   `U`.
        unsafe { Slot::new_unchecked(self.ptr.as_ptr().cast()) }
    }
}

impl<'a, T> Slot<'a, T> {
    /// Returns a new `Slot` backed by the given `MaybeUninit`.
    pub fn new(value: &'a mut MaybeUninit<T>) -> Self {
        // SAFETY:
        // - `value.as_mut_ptr()` is non-null, properly aligned, and valid for
        //   reads and writes because it is a reference to the contents of a
        //   `MaybeUninit`.
        // - `value.as_mut_ptr()` does not alias any other references for `'a`
        //   because `value` is a mutable reference.
        unsafe { Self::new_unchecked(value.as_mut_ptr()) }
    }

    /// Sets the value of the `Slot`.
    ///
    /// This overwrites any previous value without dropping it, so be careful
    /// not to use this after initializing the slot unless you want to skip
    /// running the destructor.
    ///
    /// Because this method does not convert the `Slot` to a value, the written
    /// value will not be dropped when the returned reference or underlying
    /// `Slot` leave scope.
    pub fn write(&mut self, value: T) -> &mut T {
        self.as_maybe_uninit_mut().write(value)
    }

    /// Returns a reference to the underlying memory as a `MaybeUninit`.
    pub fn as_maybe_uninit(&self) -> &MaybeUninit<T> {
        // SAFETY: `self.as_ptr()` always points to a valid `MaybeUninit<T>`
        // when `T` is `Sized`.
        unsafe { &*self.as_ptr().cast() }
    }

    /// Returns a reference to the underlying memory as a mutable `MaybeUninit`.
    pub fn as_maybe_uninit_mut(&mut self) -> &mut MaybeUninit<T> {
        // SAFETY: `self.as_ptr()` always points to a valid `MaybeUninit<T>`
        // when `T` is `Sized`.
        unsafe { &mut *self.as_ptr().cast() }
    }
}

impl<'a, T, const N: usize> Slot<'a, [T; N]> {
    /// Returns the length of the pointed-to array.
    #[inline]
    pub fn len(&self) -> usize {
        N
    }

    /// Returns whether the pointed-to array is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        N == 0
    }

    /// Gets an element of the slot array.
    #[inline]
    pub fn get(self, index: usize) -> Slot<'a, T> {
        assert!(index < self.len());
        // SAFETY: We have asserted that `index` is in-bounds.
        unsafe { self.get_unchecked(index) }
    }

    /// Gets an element of the slot array without doing bounds checking.
    ///
    /// For a safe alternative, see [`get`](Slot::get).
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior
    /// even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked(self, index: usize) -> Slot<'a, T> {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: `ptr.add(index)` will always end up inside the bounds of the
        // array because `index` is an in-bounds index.
        let ptr = unsafe { ptr.add(index) };
        // SAFETY:
        // - `ptr` is non-null, properly aligned, and valid for reads and writes
        //   because it is inside the bounds of the array.
        // - `ptr` does not alias any other accessible references for `'a`
        //   because the array never does either.
        unsafe { Slot::new_unchecked(ptr) }
    }

    /// Converts an slot array to an slot slice of the same length.
    #[inline]
    pub fn unsize(self) -> Slot<'a, [T]> {
        // SAFETY: The constructed slice pointer is non-null, properly aligned,
        // valid for reads and writes and points to `N` consecutive elements
        // because it already satisfied those conditions for `[T; N]`. It cannot
        // alias any other references for `'a` because the original array
        // pointer didn't either.
        unsafe {
            Slot::new_unchecked(slice_from_raw_parts_mut(
                self.ptr.as_ptr().cast::<T>(),
                N,
            ))
        }
    }
}

impl<'a, T> Slot<'a, [T]> {
    /// Returns the length of the pointed-to slice.
    pub fn len(&self) -> usize {
        ::ptr_meta::metadata(self.as_ptr())
    }

    /// Returns whether the pointed-to slice is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets an element of the slot slice.
    pub fn get(self, index: usize) -> Slot<'a, T> {
        assert!(index < self.len());
        // SAFETY: We have asserted that `index` is in-bounds.
        unsafe { self.get_unchecked(index) }
    }

    /// Gets an element of the slot slice without doing bounds checking.
    ///
    /// For a safe alternative, see [`get`](Slot::get).
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior
    /// even if the resulting reference is not used.
    pub unsafe fn get_unchecked(self, index: usize) -> Slot<'a, T> {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: `ptr.add(index)` will always end up inside the bounds of the
        // slice because `index` is an in-bounds index.
        let ptr = unsafe { ptr.add(index) };
        // SAFETY:
        // - `ptr` is non-null, properly aligned, and valid for reads and writes
        //   because it is inside the bounds of the slice.
        // - `ptr` does not alias any other accessible references for `'a`
        //   because the slice never does either.
        unsafe { Slot::new_unchecked(ptr) }
    }
}

// SAFETY: `Slot` returns the same value from `target`, `deref`, and
// `deref_mut`.
unsafe impl<T: ?Sized> Pointer for Slot<'_, T> {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.as_ptr()
    }
}

// SAFETY: `Destructure::underlying` for `Slot` returns the same pointer as
// `Pointer::target`.
unsafe impl<T: ?Sized> RestructurablePointer for Slot<'_, T> {}

// SAFETY:
// - `Slot<'a, T>` is destructured by reference, so its `Destructuring` type is
//   `Ref`.
// - `underlying` returns the pointer inside the `Slot<'a, T>`, which is
//   guaranteed to be non-null, properly-aligned, and valid for reads.
unsafe impl<'a, T: ?Sized> Destructure for Slot<'a, T> {
    type Underlying = T;
    type Destructuring = ::munge::Ref;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_mut().as_ptr()
    }
}

// SAFETY: `restructure` returns a `Slot<'a, U>` that borrows the restructured
// field because `Slot<'a, T>` is destructured by reference.
unsafe impl<'a, T, U: 'a> Restructure<U> for Slot<'a, T> {
    type Restructured = Slot<'a, U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY: `ptr` upholds the same invariants as its destructured slot,
        // so it must be non-null, properly aligned, valid for reads and writes,
        // and not alias any accessible references.
        unsafe { Slot::new_unchecked(ptr) }
    }
}

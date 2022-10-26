use ::core::{
    fmt,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::{copy_nonoverlapping, NonNull},
};
use ::mischief::{
    layout_of_val_raw,
    Metadata,
    Pointer,
    Region,
    RestructurablePointer,
    Slot,
    Unique,
    Within,
};
use ::munge::{Destructure, Restructure};
use ::ptr_meta::{metadata, Pointee};

use crate::{
    fmt::{DebugRaw, DisplayRaw},
    DropRaw,
    Mut,
    Pinned,
    Ref,
};

/// An immovable owned value in borrowed backing memory.
pub struct Val<'a, T: DropRaw + ?Sized> {
    ptr: NonNull<T>,
    _phantom: PhantomData<&'a mut T>,
}

impl<T: DropRaw + ?Sized> Drop for Val<'_, T> {
    fn drop(&mut self) {
        // SAFETY:
        // - `self.ptr` is always non-null, properly aligned, and valid for
        //   reading and writing.
        // - `Val` owns the value it points to, so `self.ptr` is always valid
        //   for dropping.
        unsafe { T::drop_raw(self.as_mut()) }
    }
}

impl<'a, T: DropRaw + ?Sized> Val<'a, T> {
    /// Creates a new `Val` from an exclusively owned pointer.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null, properly aligned, and valid for reading,
    ///   writing, and dropping.
    /// - `ptr` must not alias any other accessible references for `'a`.
    /// - The value pointed to by `ptr` must be initialized and immovable.
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            // SAFETY: `ptr` is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            _phantom: PhantomData,
        }
    }

    /// Creates a new `Val` from an initialized `Slot`.
    ///
    /// # Safety
    ///
    /// The value pointed to by `slot` must be initialized, valid for dropping,
    /// and immovable.
    pub unsafe fn from_slot_unchecked(slot: Slot<'a, T>) -> Self {
        Self {
            // SAFETY: `Slot`s always have a non-null pointer.
            ptr: unsafe { NonNull::new_unchecked(slot.as_ptr()) },
            _phantom: PhantomData,
        }
    }

    /// Returns a pointer to the referenced value.
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Consumes the `Val` and leaks its value, returning a mutable reference
    /// `&'a mut T`.
    ///
    /// This function is mainly useful for data that lives for as long as its
    /// backing memory. Dropping the returned reference will cause a memory
    /// leak. If this is not acceptable, then the reference should first be
    /// wrapped with the [`Val::new_unchecked`] function, producing a `Val`.
    /// This `Val` can then be dropped which will properly destroy `T`.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Val::leak(x)` instead of `x.leak()`. This is so that there is no
    /// conflict with a method named `leak` on the value type.
    pub fn leak(v: Self) -> Mut<'a, T> {
        let v = ManuallyDrop::new(v);

        // SAFETY:
        // - `v.ptr` is always non-null, properly aligned, and valid for reads
        //   and writes.
        // - `v.ptr` always points to a valid `T` and does not alias any other
        //   mutable references because it has the same aliasing as `v`.
        // - The value pointed to by `v.ptr` will live for at least `'a` because
        //   its backing memory lives for at least that long. `v` was the only
        //   site allowed to drop the value, so it will also not be dropped.
        // - `v` ensured that the pointee of `v.ptr` is immovable, so creating
        //   a `Mut` out of it will continue that guarantee.
        unsafe { Mut::new_unchecked(v.ptr.as_ptr()) }
    }

    /// Casts a `Val<T>` to a `Val<U>`.
    ///
    /// # Safety
    ///
    /// The value owned by `self` must be a valid `U`.
    pub unsafe fn cast<U: DropRaw>(self) -> Val<'a, U>
    where
        T: Sized,
    {
        let v = ManuallyDrop::new(self);

        // SAFETY: `v.ptr` is always non-null, properly aligned, and valid for
        // reading, writing, and dropping. The caller has guaranteed that `self`
        // actually owns a `U`, so the cast `Val` is also initialized and
        // immovable.
        unsafe { Val::new_unchecked(v.ptr.as_ptr().cast()) }
    }

    /// Consumes the `Val` and returns the value it contained.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Val::read(this)` instead of `this.read()`. This is so that there
    /// is no conflict with a method on the inner type.
    pub fn read(this: Self) -> T
    where
        T: Sized + Unpin,
    {
        let this = ManuallyDrop::new(this);
        // SAFETY: `ptr` is always non-null, properly aligned, and valid for
        // reads. The value may be moved because it implements `Unpin`.
        unsafe { this.ptr.as_ptr().read() }
    }

    /// Consumes the `Val` and moves it into the given `Slot`.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Val::read_unsized(this, slot)` instead of
    /// `this.read_unsized(slot)`. This is so that there is no conflict with a
    /// method on the inner type.
    ///
    /// # Panics
    ///
    /// Panics if `slot` does not have the same metadata as `this`.
    pub fn read_unsized(this: Self, slot: Slot<'_, T>)
    where
        T: Pointee + Unpin,
        <T as Pointee>::Metadata: Metadata<T>,
    {
        assert!(metadata(this.as_ptr()) == metadata(slot.as_ptr()));
        // SAFETY: We asserted that `this` has the same metadata as `slot`.
        unsafe {
            Self::read_unsized_unchecked(this, slot);
        }
    }

    /// Consumes the `Val` and moves it into the given `Slot`.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `Val::read_unsized_unchecked(this, slot)` instead of
    /// `this.read_unsized_unchecked(slot)`. This is so that there is no
    /// conflict with a method on the inner type.
    ///
    /// # Safety
    ///
    /// `slot` must have the same metadata as `this`.
    pub unsafe fn read_unsized_unchecked(this: Self, slot: Slot<'_, T>)
    where
        T: Pointee + Unpin,
        <T as Pointee>::Metadata: Metadata<T>,
    {
        let this = ManuallyDrop::new(this);
        let layout = layout_of_val_raw(this.as_ptr());
        // SAFETY:
        // - `this` is valid for reads of `size` bytes because that is the size
        unsafe {
            copy_nonoverlapping(
                this.as_ptr().cast::<u8>(),
                slot.as_ptr().cast::<u8>(),
                layout.size(),
            );
        }
    }

    /// Forgets the contained value, returning a `Slot` of the underlying
    /// memory.
    pub fn forget(this: Self) -> Slot<'a, T> {
        let this = ManuallyDrop::new(this);
        // SAFETY: `ptr` is a valid pointer for `'a` and the returned `Slot` is
        // borrowed for `'b` and cannot be modified until the returned value is
        // dropped.
        unsafe { Slot::new_unchecked(this.ptr.as_ptr()) }
    }

    /// Drops the contained value, returning a `Slot` of the underlying memory.
    pub fn drop(this: Self) -> Slot<'a, T> {
        // SAFETY: `ptr` is a valid pointer for `'a` and the returned `Slot` is
        // borrowed for `'b` and cannot be modified until the returned value is
        // dropped.
        let result = unsafe { Slot::new_unchecked(this.ptr.as_ptr()) };
        drop(this);
        result
    }

    /// Returns a `Ref` of the referenced value.
    pub fn as_ref(&self) -> Ref<'_, T> {
        // SAFETY: The requirements for `Ref` are a subset of those for `Val`.
        unsafe { Ref::new_unchecked(self.as_ptr()) }
    }

    /// Returns a reborrowed `Mut` of the referenced value.
    pub fn as_mut(&mut self) -> Mut<'_, T> {
        // SAFETY: The requirements for `Ref` are a subset of those for `Val`.
        unsafe { Mut::new_unchecked(self.as_ptr()) }
    }
}

// SAFETY: `Val` returns the same value from `target`, `deref`, and `deref_mut`.
unsafe impl<T: DropRaw + ?Sized> Pointer for Val<'_, T> {
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        self.ptr.as_ptr()
    }
}

// SAFETY: `T` is only located in `R`, so the targets of all `Val<'_, T>` must
// be located in `R`.
unsafe impl<T, R> Within<R> for Val<'_, T>
where
    T: DropRaw + Pinned<R> + ?Sized,
    R: Region,
{
}

impl<T: DropRaw + ?Sized> Deref for Val<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `Val<'a, T>` lives for `'a` at most, the lifetime of
        //   `&self` must be shorter than `'a`. That lifetime is used for the
        //   returned reference, so the returned reference is valid for `'a` and
        //   has shared read-only aliasing.
        unsafe { self.ptr.as_ref() }
    }
}

// Note that `T` must be `Unpin` to avoid violating the immovability invariant
// of `Val`.
impl<T: DropRaw + Unpin + ?Sized> DerefMut for Val<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `Val<'a, T>` is mutably borrowed for `'_`, the returned
        //   reference is also valid for `'_` and has unique read-write
        //   aliasing.
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<T: DebugRaw + DropRaw + ?Sized> fmt::Debug for Val<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugRaw::fmt_raw(self.as_ref(), f)
    }
}

impl<T: DisplayRaw + DropRaw + ?Sized> fmt::Display for Val<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayRaw::fmt_raw(self.as_ref(), f)
    }
}

// SAFETY: `Destructure::underlying` for `Val` returns the same pointer as
// `Pointer::target`.
unsafe impl<T: DropRaw + ?Sized> RestructurablePointer for Val<'_, T> {}

// SAFETY:
// - `Val<'a, T>` is destructured by value, so its `Destructuring` type is
//   `Value`.
// - `underlying` returns the pointer inside the `Val<'a, T>`, which is
//   guaranteed to be non-null, properly-aligned, and valid for reads.
unsafe impl<'a, T: DropRaw + ?Sized> Destructure for Val<'a, T> {
    type Underlying = T;
    type Destructuring = ::munge::Value;

    fn underlying(&mut self) -> *mut Self::Underlying {
        self.as_ptr()
    }
}

// SAFETY: `restructure` returns a `Val<'a, U>` that takes ownership of the
// restructured field because `Val<'a, T>` is destructured by value.
unsafe impl<'a, T: DropRaw, U: 'a + DropRaw> Restructure<U> for Val<'a, T> {
    type Restructured = Val<'a, U>;

    unsafe fn restructure(&self, ptr: *mut U) -> Self::Restructured {
        // SAFETY:
        // - A pointer to a subfield of a `Val` is also non-null, properly
        //   aligned, and valid for reads and writes. It is also valid for
        //   dropping because we have been given ownership of the field.
        // - `munge` enforces that the field pointer cannot alias another
        //   accessible reference to the field. Because `Val` owns the entire
        //   object, there cannot be another mutable reference to one of its
        //   fields.
        // - All of the fields of a `Val<'a, T>` must be initialized and
        //   immovable because the overall `Val<'a, T>` is initialized and
        //   immovable.
        unsafe { Val::new_unchecked(ptr) }
    }
}

// SAFETY: Because the `T` value is unique and values can only have one owner,
// there can only ever be one `Val` of each unique `T` at any time.
unsafe impl<T: DropRaw + Unique + ?Sized> Unique for Val<'_, T> {}

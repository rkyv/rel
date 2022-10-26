//! A pointer type for memory allocation.

use ::core::{
    alloc::Layout,
    fmt,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use ::heresy::alloc::Allocator;
#[cfg(feature = "alloc")]
use ::heresy::alloc::Global;
use ::mischief::{Frame, Metadata, Pointer, RegionalAllocator, Within};
use ::ptr_meta::Pointee;

use crate::{
    fmt::{DebugRaw, DisplayRaw},
    DropRaw,
    Mut,
    Ref,
};

/// A pointer type for memory allocation.
pub struct OwnedVal<
    T: DropRaw + ?Sized,
    #[cfg(feature = "alloc")] A: Allocator = Global,
    #[cfg(not(feature = "alloc"))] A: Allocator,
> {
    ptr: NonNull<T>,
    alloc: A,
}

impl<T: DropRaw + ?Sized, A: Allocator> Drop for OwnedVal<T, A> {
    fn drop(&mut self) {
        let layout = Layout::for_value(&**self);
        // SAFETY: `self.ptr` is always valid for reads and writes, properly
        // aligned, and valid for dropping.
        unsafe {
            T::drop_raw(self.as_mut());
        }
        // SAFETY: `self.ptr` is always allocated via `self.alloc` and `layout`
        // is guaranteed to fit it.
        unsafe {
            self.alloc.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T: DropRaw + ?Sized, A: Allocator> OwnedVal<T, A> {
    /// Returns a reference to the underlying allocator.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `OwnedVal::allocator(&v)` instead of `owned_val.allocator()`. This
    /// is so that there is no conflict with a method on the inner type.
    pub fn allocator(v: &Self) -> &A {
        &v.alloc
    }

    /// Constructs an owned `Val` from a raw pointer in the given allocator.
    ///
    /// After calling this function, the raw pointer is owned by the resulting
    /// `OwnedVal`. Specifically, the `OwnedVal` destructor will call the
    /// `DropRaw` destructor of `T` and free the allocated memory. For this to
    /// be safe, the memory must have been allocated in accordance with the
    /// memory layout used by `OwnedVal`.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to a memory block currently allocated by `alloc`.
    /// - The layout used to allocate `ptr` must exactly match the return value
    ///   of `Layout::for_value`.
    /// - `ptr` must point to an initialized `T`.
    pub unsafe fn from_raw_in(ptr: *mut T, alloc: A) -> Self {
        Self {
            // SAFETY: `ptr` is allocated by `alloc`, and so must not be null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            alloc,
        }
    }

    /// Constructs an owned `Val` from an initialized `Frame`.
    ///
    /// # Safety
    ///
    /// `frame` must be initialized.
    pub unsafe fn assume_init(frame: Frame<T, A>) -> Self
    where
        T: Pointee,
        <T as Pointee>::Metadata: Metadata<T>,
    {
        let (ptr, alloc) = Frame::into_raw_with_allocator(frame);
        // SAFETY:
        // - `Frame`'s `ptr` is always allocated in its returned `alloc` with
        //   the layout returned from `Metadata::pointee_layout` (which is
        //   equivalent to `Layout::for_value`).
        // - The caller has guaranteed that `frame`'s pointer is initialized.
        unsafe { Self::from_raw_in(ptr, alloc) }
    }

    /// Consumes the `OwnedVal`, returning a wrapped raw pointer and the
    /// allocator.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `OwnedVal`. In particular, the caller should
    /// properly destroy `T` with `DropRaw` and release the memory, taking into
    /// account the memory layout used by `OwnedVal`. The easiest way to do this
    /// is to convert the raw pointer back into an `OwnedVal` with the
    /// `OwnedVal::from_raw_in` function, allowing the `OwnedVal` destructor to
    /// perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `OwnedVal::into_raw(b)` instead of `b.into_raw()`. This is so that
    /// there is no conflict with a method on the inner type.
    pub fn into_raw_parts(b: Self) -> (*mut T, A) {
        let b = ManuallyDrop::new(b);
        // SAFETY: `b.alloc` is a valid `A` that is moved into the return value
        // and not dropped.
        let alloc = unsafe { ::core::ptr::read(&b.alloc) };
        (b.ptr.as_ptr(), alloc)
    }

    /// Returns a `Ref` of the owned value.
    pub fn as_ref(&self) -> Ref<'_, T> {
        // SAFETY: `self.ptr` is always non-null, properly aligned, and valid
        // for reads. Because `self` owns the value and it is borrowed, the
        // value pointed to by `self.ptr` may only be aliased by shared
        // references. And finally, it's an invariant of `OwnedVal` that the
        // value pointed to is initialized.
        unsafe { Ref::new_unchecked(self.ptr.as_ptr()) }
    }

    /// Returns a `Mut` of the owned value.
    pub fn as_mut(&mut self) -> Mut<'_, T> {
        // SAFETY: `self.ptr` is always non-null, properly aligned, and valid
        // for reads. Because `self` owns the value and it is mutably borrowed,
        // the value pointed to by `self.ptr` may not be aliased. And finally,
        // it's an invariant of `OwnedVal` that the value pointed to is
        // initialized.
        unsafe { Mut::new_unchecked(self.ptr.as_ptr()) }
    }
}

impl<T: DropRaw, A: Allocator> OwnedVal<T, A> {
    /// Allocates memory in the given allocator then places `x` into it.
    pub fn new_in(x: T, alloc: A) -> OwnedVal<T, A> {
        let mut frame = Frame::new_in(alloc);
        frame.write(x);
        // SAFETY: We initialized `frame` by writing to it.
        unsafe { Self::assume_init(frame) }
    }
}

#[cfg(feature = "alloc")]
impl<T: DropRaw> OwnedVal<T, Global> {
    /// Allocates memory in the `Global` allocator and then places `x` into it.
    pub fn new(x: T) -> Self {
        Self::new_in(x, Global)
    }
}

// SAFETY: `OwnedVal` returns the same value from `target`, `deref`, and
// `deref_mut`.
unsafe impl<T, A> Pointer for OwnedVal<T, A>
where
    T: DropRaw + Pointee + ?Sized,
    A: Allocator,
{
    type Target = T;

    fn target(&self) -> *mut Self::Target {
        OwnedVal::as_ref(self).as_ptr()
    }
}

// SAFETY: The pointee of `OwnedVal<T, A>` is located in `A::Region` because it
// is allocated in `A`.
unsafe impl<T, A> Within<A::Region> for OwnedVal<T, A>
where
    T: DropRaw + Pointee + ?Sized,
    A: RegionalAllocator,
{
}

impl<T: DropRaw + ?Sized, A: Allocator> Deref for OwnedVal<T, A> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `OwnedVal<T, A>` is borrowed for `'_`, the returned
        //   reference is also valid for `'_` and has shared read-only aliasing.
        unsafe { &*self.ptr.as_ptr() }
    }
}

// Note that `T` must be `Unpin` to avoid violating the immovability invariant
// of `OwnedVal`.
impl<T: DropRaw + Unpin + ?Sized, A: Allocator> DerefMut for OwnedVal<T, A> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // - `self.ptr` is always properly aligned and dereferenceable.
        // - `self.ptr` always points to an initialized value of `T`.
        // - Because `OwnedVal<T, A>` is mutably borrowed for `'_`, the returned
        //   reference is also valid for `'_` and has unique read-write
        //   aliasing.
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<T, A> fmt::Debug for OwnedVal<T, A>
where
    T: DebugRaw + DropRaw + ?Sized,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugRaw::fmt_raw(self.as_ref(), f)
    }
}

impl<T, A> fmt::Display for OwnedVal<T, A>
where
    T: DisplayRaw + DropRaw + ?Sized,
    A: Allocator,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayRaw::fmt_raw(self.as_ref(), f)
    }
}
